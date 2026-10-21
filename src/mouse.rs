//! Optional global middle-button trigger. The hook thread only filters input and
//! posts a message; scanning, drawing and application state stay on the UI thread.
use std::{
    cell::RefCell,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
};
use windows::{
    Win32::{
        Foundation::{HWND, LPARAM, LRESULT, WPARAM},
        System::{LibraryLoader::GetModuleHandleW, Threading::GetCurrentThreadId},
        UI::WindowsAndMessaging::*,
    },
    core::Result,
};

pub const WM_MIDDLE_CLICK: u32 = WM_APP + 13;

struct State {
    target: isize,
    enabled: Arc<AtomicBool>,
    click: Click,
}

#[derive(Default)]
struct Click {
    captured: bool,
}

impl Click {
    /// (consume this edge, toggle the panel). Never consume an orphan release,
    /// and never leak a captured release if the setting changed mid-click.
    fn event(&mut self, down: bool, enabled: bool) -> (bool, bool) {
        if down {
            self.captured = enabled;
            (enabled, false)
        } else {
            let captured = std::mem::take(&mut self.captured);
            (captured, captured && enabled)
        }
    }
}

thread_local! {
    static STATE: RefCell<Option<State>> = const { RefCell::new(None) };
}

pub struct MiddleMouse {
    thread: u32,
    enabled: Arc<AtomicBool>,
}

impl MiddleMouse {
    pub fn start(hwnd: HWND, on: bool) -> Result<Self> {
        let target = hwnd.0 as isize;
        let enabled = Arc::new(AtomicBool::new(on));
        let flag = enabled.clone();
        let (tx, rx) = mpsc::sync_channel(1);
        std::thread::spawn(move || unsafe {
            STATE.with(|s| {
                *s.borrow_mut() = Some(State {
                    target,
                    enabled: flag,
                    click: Click::default(),
                })
            });
            let hook = GetModuleHandleW(None).and_then(|module| {
                SetWindowsHookExW(WH_MOUSE_LL, Some(mouse_proc), Some(module.into()), 0)
            });
            match hook {
                Ok(hook) => {
                    // Create the message queue before reporting the thread id,
                    // so Drop can always post WM_QUIT, even immediately.
                    let mut msg = MSG::default();
                    let _ = PeekMessageW(&mut msg, None, 0, 0, PM_NOREMOVE);
                    if tx.send(Ok(GetCurrentThreadId())).is_ok() {
                        while GetMessageW(&mut msg, None, 0, 0).0 > 0 {}
                    }
                    let _ = UnhookWindowsHookEx(hook);
                }
                Err(e) => {
                    let _ = tx.send(Err(e));
                }
            }
            STATE.with(|s| *s.borrow_mut() = None);
        });
        let thread = rx.recv().map_err(|_| {
            windows::core::Error::new(
                windows::Win32::Foundation::E_FAIL,
                "Mouse hook thread stopped",
            )
        })??;
        Ok(Self { thread, enabled })
    }

    pub fn set_enabled(&self, on: bool) {
        self.enabled.store(on, Ordering::Relaxed);
    }
}

impl Drop for MiddleMouse {
    fn drop(&mut self) {
        self.set_enabled(false);
        unsafe {
            let _ = PostThreadMessageW(self.thread, WM_QUIT, WPARAM(0), LPARAM(0));
        }
    }
}

unsafe extern "system" fn mouse_proc(code: i32, wp: WPARAM, lp: LPARAM) -> LRESULT {
    if code == HC_ACTION as i32 && matches!(wp.0 as u32, WM_MBUTTONDOWN | WM_MBUTTONUP) {
        let data = unsafe { &*(lp.0 as *const MSLLHOOKSTRUCT) };
        // Do not turn synthesized mouse input into another launch gesture.
        if data.flags & LLMHF_INJECTED == 0 {
            let consumed = STATE.with(|cell| {
                let mut state = cell.borrow_mut();
                let Some(s) = state.as_mut() else {
                    return false;
                };
                let (consume, toggle) = s.click.event(
                    wp.0 as u32 == WM_MBUTTONDOWN,
                    s.enabled.load(Ordering::Relaxed),
                );
                if toggle {
                    unsafe {
                        let _ = PostMessageW(
                            Some(HWND(s.target as _)),
                            WM_MIDDLE_CLICK,
                            WPARAM(0),
                            LPARAM(0),
                        );
                    }
                }
                consume
            });
            if consumed {
                return LRESULT(1);
            }
        }
    }
    unsafe { CallNextHookEx(None, code, wp, lp) }
}

#[cfg(test)]
mod tests {
    use super::Click;

    #[test]
    fn only_a_complete_captured_click_toggles_once() {
        let mut click = Click::default();
        assert_eq!(click.event(false, true), (false, false));
        assert_eq!(click.event(true, true), (true, false));
        assert_eq!(click.event(false, true), (true, true));
        assert_eq!(click.event(false, true), (false, false));
        assert_eq!(click.event(true, false), (false, false));
        assert_eq!(click.event(false, false), (false, false));
    }

    #[test]
    fn changing_the_switch_mid_click_preserves_down_up_pairing() {
        let mut click = Click::default();
        assert_eq!(click.event(true, true), (true, false));
        assert_eq!(click.event(false, false), (true, false));
        assert_eq!(click.event(true, false), (false, false));
        assert_eq!(click.event(false, true), (false, false));
    }
}
