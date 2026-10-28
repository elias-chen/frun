//! Input service: long-press recognition, foreground exclusions and global hotkey.
//! All hooks and timers live on one message thread; no drawing or scanning runs here.
use crate::{
    config, desktop,
    gesture::{self, Action, Hold, Point},
};
use std::{
    cell::RefCell,
    sync::{Arc, Mutex, atomic::{AtomicUsize, Ordering}, mpsc},
    time::{Duration, Instant},
};
use windows::{
    Win32::{
        Foundation::*,
        System::{
            LibraryLoader::GetModuleHandleW,
            Threading::{GetCurrentProcessId, GetCurrentThreadId},
        },
        UI::{Accessibility::*, Input::KeyboardAndMouse::*, WindowsAndMessaging::*},
    },
    core::Result,
};

pub const WM_MIDDLE_CLICK: u32 = WM_APP + 13;
pub const WM_WAKE_KEY: u32 = WM_APP + 14;
pub const WM_WAKE_STATUS: u32 = WM_APP + 15;
const WM_CONFIG: u32 = WM_APP + 16;
pub const WM_OUTSIDE_CLICK: u32 = WM_APP + 17;
const HOTKEY: i32 = 1;

#[derive(Clone, Default)]
pub struct Settings {
    pub middle: bool,
    pub hold_ms: u32,
    pub paused: bool,
    pub hotkey: Option<(u32, u32)>,
    pub excluded: Vec<String>,
}
impl Settings {
    pub fn from_config(cfg: &config::Config, paused: bool, recording: bool) -> Self {
        Self {
            middle: cfg.middle_mouse,
            hold_ms: cfg.hold_ms,
            paused,
            hotkey: if recording {
                None
            } else {
                config::parse_hotkey(&cfg.hotkey)
            },
            excluded: cfg.excluded_apps.clone(),
        }
    }
    fn blocked(&self, exe: &str) -> bool {
        self.paused || gesture::excluded(exe, &self.excluded)
    }
}

#[derive(Clone, Default, PartialEq)]
pub struct Status {
    pub paused: bool,
    pub excluded: String,
    pub hotkey_ok: bool,
    pub mouse_ok: bool,
}
type Request = (Settings, mpsc::SyncSender<bool>);

pub struct MiddleMouse {
    thread: u32,
    requests: mpsc::Sender<Request>,
    status: Arc<Mutex<Status>>,
    click_watch: Arc<ClickWatch>,
}

/// A generation guards against clicks queued before a hide and subsequent show.
/// The low bit indicates that a visible panel is being watched.
#[derive(Default)]
struct ClickWatch(AtomicUsize);
impl ClickWatch {
    fn set(&self, active: bool) {
        let old = self.0.load(Ordering::Relaxed);
        let next = (old.wrapping_add(2) & !1) | usize::from(active);
        self.0.store(next, Ordering::Release);
    }
    fn outside(&self, generation: usize, point: POINT, bounds: RECT) -> Option<usize> {
        let outside = point.x < bounds.left || point.x >= bounds.right
            || point.y < bounds.top || point.y >= bounds.bottom;
        (outside && self.current(generation)).then_some(generation)
    }
    fn snapshot(&self) -> Option<usize> {
        let generation = self.0.load(Ordering::Acquire);
        (generation & 1 != 0).then_some(generation)
    }
    fn current(&self, generation: usize) -> bool {
        generation & 1 != 0 && self.0.load(Ordering::Acquire) == generation
    }
}

struct State {
    target: isize,
    settings: Settings,
    status: Arc<Mutex<Status>>,
    click_watch: Arc<ClickWatch>,
    hold: Hold,
    press_window: isize,
    foreground: isize,
    exe: String,
    registered: Option<(u32, u32)>,
    mouse_ok: bool,
    hold_timer: usize,
    clock: Instant,
}
thread_local! { static STATE: RefCell<Option<State>> = const { RefCell::new(None) }; }

impl MiddleMouse {
    pub fn start(hwnd: HWND, settings: Settings) -> Result<Self> {
        let target = hwnd.0 as isize;
        let status = Arc::new(Mutex::new(Status::default()));
        let shared = status.clone();
        let click_watch = Arc::new(ClickWatch::default());
        let watched = click_watch.clone();
        let (requests, rx) = mpsc::channel::<Request>();
        let (ready, started) = mpsc::sync_channel(1);
        std::thread::spawn(move || unsafe {
            let mut msg = MSG::default();
            let _ = PeekMessageW(&mut msg, None, 0, 0, PM_NOREMOVE);
            let mouse = GetModuleHandleW(None)
                .and_then(|m| SetWindowsHookExW(WH_MOUSE_LL, Some(mouse_proc), Some(m.into()), 0));
            if let Err(e) = &mouse {
                crate::scan::log(&format!("mouse hook: {e}"));
            }
            STATE.with(|s| {
                *s.borrow_mut() = Some(State {
                    target,
                    settings,
                    status: shared,
                    click_watch: watched,
                    hold: Hold::default(),
                    press_window: 0,
                    foreground: 0,
                    exe: String::new(),
                    registered: None,
                    mouse_ok: mouse.is_ok(),
                    hold_timer: 0,
                    clock: Instant::now(),
                })
            });
            let foreground_hook = SetWinEventHook(
                EVENT_SYSTEM_FOREGROUND,
                EVENT_SYSTEM_FOREGROUND,
                None,
                Some(foreground_proc),
                0,
                0,
                WINEVENT_OUTOFCONTEXT,
            );
            // A low-frequency fallback also handles unusual focus notifications.
            let heartbeat = SetTimer(None, 0, 250, None);
            with_state(|s| {
                s.refresh();
            });
            if ready.send(GetCurrentThreadId()).is_ok() {
                while GetMessageW(&mut msg, None, 0, 0).0 > 0 {
                    if msg.message == WM_CONFIG {
                        while let Ok((settings, ack)) = rx.try_recv() {
                            let mut ok = false;
                            with_state(|s| {
                                s.hold.cancel();
                                s.stop_timer();
                                s.settings = settings;
                                s.refresh();
                                ok = s.status.lock().map(|v| v.hotkey_ok).unwrap_or(false);
                            });
                            let _ = ack.send(ok);
                        }
                    } else if msg.message == WM_HOTKEY {
                        with_state(|s| {
                            s.refresh();
                            if !s.settings.blocked(&s.exe) {
                                let _ = PostMessageW(
                                    Some(HWND(s.target as _)),
                                    WM_WAKE_KEY,
                                    WPARAM(0),
                                    LPARAM(0),
                                );
                            }
                        });
                    } else if msg.message == WM_TIMER {
                        with_state(|s| {
                            if msg.wParam.0 == heartbeat {
                                s.refresh();
                            }
                            s.tick();
                        });
                    } else {
                        let _ = DispatchMessageW(&msg);
                    }
                }
            }
            with_state(|s| {
                s.stop_timer();
                let _ = UnregisterHotKey(None, HOTKEY);
                if s.hold.pending() && GetForegroundWindow().0 as isize == s.press_window {
                    // Exiting during a pending physical press must not lose its down edge.
                    replay(s.hold.origin, false);
                }
            });
            let _ = KillTimer(None, heartbeat);
            if !foreground_hook.is_invalid() {
                let _ = UnhookWinEvent(foreground_hook);
            }
            if let Ok(hook) = mouse {
                let _ = UnhookWindowsHookEx(hook);
            }
            STATE.with(|s| *s.borrow_mut() = None);
        });
        let thread = started
            .recv()
            .map_err(|_| windows::core::Error::new(E_FAIL, "Input service stopped"))?;
        Ok(Self {
            thread,
            requests,
            status,
            click_watch,
        })
    }
    pub fn configure(&self, settings: Settings) -> bool {
        let (ack, rx) = mpsc::sync_channel(1);
        if self.requests.send((settings, ack)).is_err() {
            return false;
        }
        unsafe {
            if PostThreadMessageW(self.thread, WM_CONFIG, WPARAM(0), LPARAM(0)).is_err() {
                return false;
            }
        }
        rx.recv_timeout(Duration::from_secs(1)).unwrap_or(false)
    }
    pub fn status(&self) -> Status {
        self.status.lock().map(|s| s.clone()).unwrap_or_default()
    }
    pub fn watch_clicks(&self, active: bool) {
        self.click_watch.set(active);
    }
    pub fn current_click(&self, generation: usize) -> bool {
        self.click_watch.current(generation)
    }
}
impl Drop for MiddleMouse {
    fn drop(&mut self) {
        unsafe {
            let _ = PostThreadMessageW(self.thread, WM_QUIT, WPARAM(0), LPARAM(0));
        }
    }
}
fn with_state(f: impl FnOnce(&mut State)) {
    STATE.with(|cell| {
        if let Ok(mut state) = cell.try_borrow_mut() {
            if let Some(s) = state.as_mut() {
                f(s);
            }
        }
    });
}
impl State {
    fn stop_timer(&mut self) {
        if self.hold_timer != 0 {
            unsafe {
                let _ = KillTimer(None, self.hold_timer);
            }
            self.hold_timer = 0;
        }
    }
    fn refresh(&mut self) {
        if let Some(w) = desktop::foreground() {
            if self.foreground != w.hwnd {
                self.hold.cancel();
                self.stop_timer();
                self.foreground = w.hwnd;
                desktop::remember(HWND(w.hwnd as _));
            }
            self.exe = w.exe;
        } else {
            // Manual access to frun's tray/settings is its own foreground
            // context; do not retain the excluded game as the active app.
            unsafe {
                let hwnd = GetForegroundWindow();
                let mut pid = 0;
                GetWindowThreadProcessId(hwnd, Some(&mut pid));
                if !hwnd.is_invalid() && pid == GetCurrentProcessId() {
                    if self.foreground != hwnd.0 as isize {
                        self.hold.cancel();
                        self.stop_timer();
                    }
                    self.foreground = hwnd.0 as isize;
                    self.exe.clear();
                }
            }
        }
        let blocked = self.settings.blocked(&self.exe);
        let wanted = if blocked { None } else { self.settings.hotkey };
        if self.registered != wanted {
            unsafe {
                let _ = UnregisterHotKey(None, HOTKEY);
            }
            self.registered = None;
        }
        let mut hotkey_ok = true;
        if let Some((mods, vk)) = wanted {
            if self.registered.is_none() {
                hotkey_ok = unsafe {
                    RegisterHotKey(None, HOTKEY, HOT_KEY_MODIFIERS(mods) | MOD_NOREPEAT, vk).is_ok()
                };
                if hotkey_ok {
                    self.registered = wanted;
                }
            }
        }
        let status = Status {
            paused: self.settings.paused,
            mouse_ok: self.mouse_ok,
            hotkey_ok,
            excluded: if gesture::excluded(&self.exe, &self.settings.excluded) {
                gesture::exe_name(&self.exe).unwrap_or_default()
            } else {
                String::new()
            },
        };
        if let Ok(mut shared) = self.status.lock() {
            if *shared != status {
                *shared = status;
                unsafe {
                    let _ = PostMessageW(
                        Some(HWND(self.target as _)),
                        WM_WAKE_STATUS,
                        WPARAM(0),
                        LPARAM(0),
                    );
                }
            }
        }
    }
    fn tick(&mut self) {
        if !self.hold.pending() {
            self.stop_timer();
            return;
        }
        let same = unsafe { GetForegroundWindow().0 as isize == self.press_window };
        let allowed = same && self.settings.middle && !self.settings.blocked(&self.exe);
        if self.hold.tick(
            allowed,
            self.clock.elapsed().as_millis() as u64,
            self.settings.hold_ms,
        ) == Action::Show
        {
            let mut point = POINT::default();
            unsafe {
                let _ = GetCursorPos(&mut point);
                let _ = PostMessageW(
                    Some(HWND(self.target as _)),
                    WM_MIDDLE_CLICK,
                    WPARAM(point.x as u32 as usize),
                    LPARAM(point.y as isize),
                );
            }
        }
        if !self.hold.pending() {
            self.stop_timer();
        }
    }
}
unsafe extern "system" fn foreground_proc(
    _: HWINEVENTHOOK,
    _: u32,
    _: HWND,
    _: i32,
    _: i32,
    _: u32,
    _: u32,
) {
    with_state(|s| s.refresh());
}

fn notify_outside_click(hwnd: HWND, watch: &ClickWatch, point: POINT) {
    let Some(generation) = watch.snapshot() else { return };
    let mut rect = RECT::default();
    if unsafe { GetWindowRect(hwnd, &mut rect) }.is_ok() {
        if let Some(generation) = watch.outside(generation, point, rect) {
            unsafe {
                let _ = PostMessageW(Some(hwnd), WM_OUTSIDE_CLICK, WPARAM(generation), LPARAM(0));
            }
        }
    }
}

unsafe extern "system" fn mouse_proc(code: i32, wp: WPARAM, lp: LPARAM) -> LRESULT {
    if code == HC_ACTION as i32 && matches!(wp.0 as u32, WM_LBUTTONDOWN | WM_RBUTTONDOWN) {
        let data = unsafe { &*(lp.0 as *const MSLLHOOKSTRUCT) };
        with_state(|s| {
            // This is observation only: the original click always continues to its target.
            notify_outside_click(HWND(s.target as _), &s.click_watch, data.pt);
        });
    }
    if code == HC_ACTION as i32
        && matches!(wp.0 as u32, WM_MBUTTONDOWN | WM_MBUTTONUP | WM_MOUSEMOVE)
    {
        let data = unsafe { &*(lp.0 as *const MSLLHOOKSTRUCT) };
        if data.flags & LLMHF_INJECTED == 0 {
            let mut consumed = false;
            with_state(|s| {
                // Foreground notifications are asynchronous. Cancel before a
                // movement can turn a pending press into a replayed native drag.
                if s.hold.pending() && unsafe { GetForegroundWindow().0 as isize != s.press_window }
                {
                    s.hold.cancel();
                }
                let point = Point {
                    x: data.pt.x,
                    y: data.pt.y,
                };
                let action = match wp.0 as u32 {
                    WM_MBUTTONDOWN => {
                        // Refresh identity on the service thread, outside the low-level hook.
                        // A stale foreground cache must never capture the first game click.
                        let current = unsafe { GetForegroundWindow().0 as isize };
                        let allowed = s.settings.middle
                            && !s.settings.blocked(&s.exe)
                            && (current == s.foreground || current == s.target);
                        s.press_window = current;
                        let action =
                            s.hold
                                .down(allowed, point, s.clock.elapsed().as_millis() as u64);
                        if action == Action::Consume && s.hold_timer == 0 {
                            s.hold_timer = unsafe { SetTimer(None, 0, 15, None) };
                        }
                        action
                    }
                    WM_MBUTTONUP => s.hold.up(),
                    _ => s.hold.moved(point),
                };
                match action {
                    Action::Consume => consumed = true,
                    Action::ReplayClick => {
                        consumed = true;
                        if unsafe { GetForegroundWindow().0 as isize == s.press_window } {
                            replay(s.hold.origin, true);
                        }
                    }
                    Action::ReplayDown => {
                        if unsafe { GetForegroundWindow().0 as isize == s.press_window } {
                            replay(s.hold.origin, false);
                        }
                    }
                    _ => {}
                }
                if !s.hold.pending() {
                    s.stop_timer();
                }
            });
            if consumed {
                return LRESULT(1);
            }
        }
    }
    unsafe { CallNextHookEx(None, code, wp, lp) }
}

/// Replay at the original press position, restoring the current cursor in the same
/// SendInput batch. Replayed events are ignored by our hook and cannot re-trigger it.
fn replay(origin: Point, release: bool) {
    unsafe {
        let mut current = POINT::default();
        let _ = GetCursorPos(&mut current);
        let input = |flags: MOUSE_EVENT_FLAGS, x: i32, y: i32| INPUT {
            r#type: INPUT_MOUSE,
            Anonymous: INPUT_0 {
                mi: MOUSEINPUT {
                    dx: x,
                    dy: y,
                    dwFlags: flags,
                    ..Default::default()
                },
            },
        };
        let absolute = |x: i32, y: i32| {
            let left = GetSystemMetrics(SM_XVIRTUALSCREEN);
            let top = GetSystemMetrics(SM_YVIRTUALSCREEN);
            let width = GetSystemMetrics(SM_CXVIRTUALSCREEN).max(2);
            let height = GetSystemMetrics(SM_CYVIRTUALSCREEN).max(2);
            input(
                MOUSEEVENTF_MOVE | MOUSEEVENTF_ABSOLUTE | MOUSEEVENTF_VIRTUALDESK,
                (((x - left) as i64 * 65535) / (width - 1) as i64) as i32,
                (((y - top) as i64 * 65535) / (height - 1) as i64) as i32,
            )
        };
        let mut inputs = vec![
            absolute(origin.x, origin.y),
            input(MOUSEEVENTF_MIDDLEDOWN, 0, 0),
        ];
        if release {
            inputs.push(input(MOUSEEVENTF_MIDDLEUP, 0, 0));
        }
        inputs.push(absolute(current.x, current.y));
        if SendInput(&inputs, std::mem::size_of::<INPUT>() as i32) != inputs.len() as u32 {
            // No further synthesis on failure: input injection may be restricted.
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn outside_clicks_only_apply_to_the_visible_panel_generation() {
        let watch = ClickWatch::default();
        let rect = RECT { left: -900, top: 100, right: -500, bottom: 400 };
        let inside = POINT { x: -700, y: 200 };
        let outside = POINT { x: -500, y: 200 };
        assert!(watch.snapshot().is_none());
        watch.set(true);
        let first = watch.snapshot().unwrap();
        assert!(watch.outside(first, inside, rect).is_none());
        assert!(watch.outside(first, POINT { x: -900, y: 100 }, rect).is_none());
        assert_eq!(watch.outside(first, outside, rect), Some(first));
        assert!(watch.current(first));
        watch.set(false);
        assert!(!watch.current(first));
        assert!(watch.snapshot().is_none());
        watch.set(true);
        let next = watch.snapshot().unwrap();
        assert!(watch.outside(first, outside, rect).is_none());
        assert_eq!(watch.outside(next, POINT { x: -700, y: 400 }, rect), Some(next));
        assert_ne!(first, next);
        assert!(!watch.current(first));
        assert!(watch.current(next));
    }
    #[test]
    fn outside_click_is_posted_to_a_window_without_requiring_foreground_activation() {
        unsafe {
            // A hidden window cannot be foreground; a real Win32 message queue
            // exercises the same notification used by the low-level hook.
            let hwnd = CreateWindowExW(
                WINDOW_EX_STYLE(0), windows::core::w!("STATIC"), windows::core::w!(""),
                WS_POPUP, 100, 100, 200, 150, None, None, None, None,
            ).unwrap();
            let watch = ClickWatch::default();
            let mut rect = RECT::default();
            GetWindowRect(hwnd, &mut rect).unwrap();
            let inside = POINT { x: rect.left, y: rect.top };
            let outside = POINT { x: rect.right, y: rect.bottom };
            let mut msg = MSG::default();
            notify_outside_click(hwnd, &watch, outside);
            assert!(!PeekMessageW(&mut msg, Some(hwnd), WM_OUTSIDE_CLICK, WM_OUTSIDE_CLICK, PM_REMOVE).as_bool());
            watch.set(true);
            notify_outside_click(hwnd, &watch, inside);
            assert!(!PeekMessageW(&mut msg, Some(hwnd), WM_OUTSIDE_CLICK, WM_OUTSIDE_CLICK, PM_REMOVE).as_bool());
            notify_outside_click(hwnd, &watch, outside);
            assert!(PeekMessageW(&mut msg, Some(hwnd), WM_OUTSIDE_CLICK, WM_OUTSIDE_CLICK, PM_REMOVE).as_bool());
            assert!(watch.current(msg.wParam.0));
            watch.set(false);
            assert!(!watch.current(msg.wParam.0));
            DestroyWindow(hwnd).unwrap();
        }
    }
    #[test]
    fn manual_pause_has_priority_over_foreground_changes() {
        let cfg = config::Config {
            excluded_apps: vec!["cs2.exe".into()],
            ..Default::default()
        };
        let mut settings = Settings::from_config(&cfg, false, false);
        assert!(settings.blocked(r"D:\Games\CS2.exe"));
        assert!(!settings.blocked("chrome.exe"));
        assert!(!settings.blocked(""));
        settings.paused = true;
        assert!(settings.blocked("chrome.exe"));
        assert!(settings.blocked("cs2.exe"));
        assert!(settings.blocked(""));
    }

    #[test]
    fn paused_service_releases_the_global_hotkey_and_resume_reclaims_it() {
        // This checks the actual Win32 registration, not just the policy flag.
        let key = (
            config::MOD_CTRL | config::MOD_ALT | config::MOD_SHIFT,
            VK_F24.0 as u32,
        );
        let mut settings = Settings {
            middle: false,
            hold_ms: 400,
            hotkey: Some(key),
            ..Default::default()
        };
        let service = MiddleMouse::start(HWND::default(), settings.clone()).unwrap();
        assert!(service.status().hotkey_ok);
        settings.paused = true;
        assert!(service.configure(settings.clone()));
        unsafe {
            RegisterHotKey(None, 77, HOT_KEY_MODIFIERS(key.0), key.1)
                .expect("paused service must release the key");
            UnregisterHotKey(None, 77).unwrap();
        }
        settings.paused = false;
        assert!(service.configure(settings.clone()));
        unsafe {
            assert!(RegisterHotKey(None, 77, HOT_KEY_MODIFIERS(key.0), key.1).is_err());
        }
        settings.paused = true;
        assert!(service.configure(settings));
        unsafe {
            RegisterHotKey(None, 77, HOT_KEY_MODIFIERS(key.0), key.1).unwrap();
            UnregisterHotKey(None, 77).unwrap();
        }
    }
}
