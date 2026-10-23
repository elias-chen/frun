//! Foreground application identity and existing-window activation.
use crate::index::Entry;
use std::{
    collections::HashMap,
    sync::{
        Mutex, OnceLock,
        atomic::{AtomicU64, Ordering},
    },
};
use windows::{
    Win32::{
        Foundation::*,
        Graphics::Dwm::*,
        System::{
            Com::{CoTaskMemFree, StructuredStorage::PropVariantToStringAlloc},
            Diagnostics::ToolHelp::*,
            Threading::*,
        },
        UI::{Shell::PropertiesSystem::*, WindowsAndMessaging::*},
    },
    core::*,
};

#[derive(Clone, Debug)]
pub struct Window {
    pub hwnd: isize,
    pub exe: String,
    pub title: String,
}
static RECENT: OnceLock<Mutex<HashMap<isize, u64>>> = OnceLock::new();
static SEQUENCE: AtomicU64 = AtomicU64::new(1);

pub fn remember(hwnd: HWND) {
    if let Ok(mut recent) = RECENT.get_or_init(Default::default).lock() {
        if recent.len() > 512 {
            recent.clear();
        }
        recent.insert(hwnd.0 as isize, SEQUENCE.fetch_add(1, Ordering::Relaxed));
    }
}

pub fn process_path(pid: u32) -> Option<String> {
    full_process_path(pid).or_else(|| process_name(pid))
}
fn full_process_path(pid: u32) -> Option<String> {
    unsafe {
        let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid).ok()?;
        let mut buf = vec![0u16; 32768];
        let mut len = buf.len() as u32;
        let ok = QueryFullProcessImageNameW(
            handle,
            PROCESS_NAME_WIN32,
            PWSTR(buf.as_mut_ptr()),
            &mut len,
        )
        .is_ok();
        let _ = CloseHandle(handle);
        ok.then(|| String::from_utf16_lossy(&buf[..len as usize]))
    }
}

/// A process name is enough for exclusions even if its image path is protected.
fn process_name(pid: u32) -> Option<String> {
    unsafe {
        let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0).ok()?;
        let mut entry = PROCESSENTRY32W {
            dwSize: std::mem::size_of::<PROCESSENTRY32W>() as u32,
            ..Default::default()
        };
        let mut result = None;
        if Process32FirstW(snapshot, &mut entry).is_ok() {
            loop {
                if entry.th32ProcessID == pid {
                    let len = entry
                        .szExeFile
                        .iter()
                        .position(|c| *c == 0)
                        .unwrap_or(entry.szExeFile.len());
                    result = Some(String::from_utf16_lossy(&entry.szExeFile[..len]));
                    break;
                }
                if Process32NextW(snapshot, &mut entry).is_err() {
                    break;
                }
            }
        }
        let _ = CloseHandle(snapshot);
        result
    }
}

pub fn foreground() -> Option<Window> {
    unsafe {
        let hwnd = GetForegroundWindow();
        if hwnd.is_invalid() {
            return None;
        }
        let mut pid = 0;
        GetWindowThreadProcessId(hwnd, Some(&mut pid));
        if pid == GetCurrentProcessId() {
            return None;
        }
        Some(Window {
            hwnd: hwnd.0 as isize,
            exe: process_path(pid).unwrap_or_default(),
            title: title(hwnd),
        })
    }
}

fn title(hwnd: HWND) -> String {
    unsafe {
        let mut text = vec![0u16; (GetWindowTextLengthW(hwnd).max(0) as usize + 1).min(2048)];
        let len = GetWindowTextW(hwnd, &mut text);
        String::from_utf16_lossy(&text[..len.max(0) as usize])
    }
}

pub fn windows() -> Vec<Window> {
    unsafe extern "system" fn each(hwnd: HWND, lp: LPARAM) -> BOOL {
        unsafe {
            if !IsWindowVisible(hwnd).as_bool() || GetWindowTextLengthW(hwnd) == 0 {
                return true.into();
            }
            if GetWindowLongPtrW(hwnd, GWL_EXSTYLE) as u32 & WS_EX_TOOLWINDOW.0 != 0 {
                return true.into();
            }
            if GetAncestor(hwnd, GA_ROOTOWNER) != hwnd {
                return true.into();
            }
            let mut cloaked = 0u32;
            let _ = DwmGetWindowAttribute(hwnd, DWMWA_CLOAKED, &mut cloaked as *mut _ as _, 4);
            if cloaked != 0 {
                return true.into();
            }
            let mut pid = 0;
            GetWindowThreadProcessId(hwnd, Some(&mut pid));
            if pid != GetCurrentProcessId() {
                if let Some(exe) = process_path(pid) {
                    let out = &mut *(lp.0 as *mut Vec<Window>);
                    out.push(Window {
                        hwnd: hwnd.0 as isize,
                        exe,
                        title: title(hwnd),
                    });
                }
            }
        }
        true.into()
    }
    let mut out: Vec<Window> = Vec::new();
    unsafe {
        let _ = EnumWindows(Some(each), LPARAM(&mut out as *mut Vec<Window> as isize));
    }
    if let Ok(recent) = RECENT.get_or_init(Default::default).lock() {
        out.sort_by_key(|w| std::cmp::Reverse(recent.get(&w.hwnd).copied().unwrap_or(0)));
    }
    out
}

fn app_id(hwnd: HWND) -> Option<String> {
    const KEY: PROPERTYKEY = PROPERTYKEY {
        fmtid: GUID::from_u128(0x9f4c2855_9f79_4b39_a8d0_e1d42de1d5f3),
        pid: 5,
    };
    unsafe {
        let props: IPropertyStore = SHGetPropertyStoreForWindow(hwnd).ok()?;
        let value = props.GetValue(&KEY).ok()?;
        let text = PropVariantToStringAlloc(&value).ok()?;
        let result = text.to_string().ok();
        CoTaskMemFree(Some(text.0 as _));
        result.filter(|s| !s.is_empty())
    }
}

/// A shortcut with arguments represents an action, not just an application.
fn executable(target: &str, resolved: &str) -> Option<String> {
    let (path, args) = resolved.split_once('|').unwrap_or((resolved, ""));
    if !args.trim().is_empty() {
        return None;
    }
    let path = if path.is_empty() { target } else { path };
    path.to_lowercase()
        .ends_with(".exe")
        .then(|| normalize(path))
}
fn normalize(path: &str) -> String {
    path.trim_start_matches(r"\\?\")
        .replace('/', "\\")
        .to_lowercase()
}

#[derive(Debug, PartialEq)]
pub enum Reuse {
    Missing,
    Activated,
    Denied,
}

pub fn activate(entry: &Entry) -> Reuse {
    let exe = executable(&entry.target, &entry.resolved);
    let id = entry
        .target
        .strip_prefix(r"shell:AppsFolder\")
        .filter(|s| s.contains('!'));
    // Preserve action shortcuts even when their target is a packaged app.
    if entry
        .resolved
        .split_once('|')
        .is_some_and(|(_, args)| !args.trim().is_empty())
    {
        return Reuse::Missing;
    }
    if exe.is_none() && id.is_none() {
        return Reuse::Missing;
    }
    let found = windows().into_iter().find(|w| {
        exe.as_ref().is_some_and(|p| normalize(&w.exe) == *p)
            || id.is_some_and(|id| {
                app_id(HWND(w.hwnd as _)).is_some_and(|s| s.eq_ignore_ascii_case(id))
            })
    });
    let Some(window) = found else {
        return Reuse::Missing;
    };
    unsafe {
        let mut hwnd = HWND(window.hwnd as _);
        if !IsWindow(Some(hwnd)).as_bool() {
            return Reuse::Missing;
        }
        let popup = GetLastActivePopup(hwnd);
        if !popup.is_invalid() && IsWindowVisible(popup).as_bool() {
            hwnd = popup;
        }
        if IsIconic(hwnd).as_bool() {
            let _ = ShowWindowAsync(hwnd, SW_RESTORE);
        }
        if SetForegroundWindow(hwnd).as_bool() {
            return Reuse::Activated;
        }
        let info = FLASHWINFO {
            cbSize: std::mem::size_of::<FLASHWINFO>() as u32,
            hwnd,
            dwFlags: FLASHW_TRAY,
            uCount: 3,
            dwTimeout: 0,
        };
        let _ = FlashWindowEx(&info);
    }
    Reuse::Denied
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn application_reuse_preserves_shortcut_arguments_and_matches_full_paths() {
        assert_eq!(
            executable("shortcut.lnk", r"C:\Apps\Code.exe|"),
            Some(r"c:\apps\code.exe".into())
        );
        assert_eq!(
            executable("shortcut.lnk", r"C:\Apps\Code.exe|--new-window D:\project"),
            None
        );
        assert_eq!(executable("document.pdf", ""), None);
        assert_eq!(
            executable(r"D:\Tools\tool.exe", ""),
            Some(r"d:\tools\tool.exe".into())
        );
        assert_eq!(
            normalize(r"\\?\C:\Apps\Code.exe"),
            normalize("c:/apps/code.exe")
        );
    }
}
