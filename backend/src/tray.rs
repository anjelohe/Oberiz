#![allow(unsafe_op_in_unsafe_fn)]

//! Windows notification-area integration.
//!
//! Two independent uses share this one implementation:
//! - `spawn()`: embedded in the portable/default process itself. "Exit"
//!   quits that same process, exactly like before the Windows Service existed.
//! - `run_standalone()`: a separate, unprivileged helper (`Oberiz.exe --tray`)
//!   started at user logon alongside the real Windows Service. It never runs
//!   the web server itself — its menu starts/stops the service instead, since
//!   a service has no desktop session to show a tray icon from.

use std::{
    iter,
    os::windows::ffi::OsStrExt,
    path::Path,
    ptr,
    sync::atomic::{AtomicBool, Ordering},
};

use windows_sys::Win32::{
    Foundation::{
        CloseHandle, ERROR_ALREADY_EXISTS, GetLastError, HANDLE, HWND, LPARAM, LRESULT, WPARAM,
    },
    System::Threading::CreateMutexW,
    UI::{
        Shell::{
            NIF_ICON, NIF_MESSAGE, NIF_TIP, NIM_ADD, NIM_DELETE, NIM_SETVERSION,
            NOTIFYICON_VERSION_4, NOTIFYICONDATAW, Shell_NotifyIconW, ShellExecuteW,
        },
        WindowsAndMessaging::{
            AppendMenuW, CreatePopupMenu, CreateWindowExW, DefWindowProcW, DestroyMenu,
            DispatchMessageW, GetCursorPos, GetMessageW, IMAGE_ICON, LR_LOADFROMFILE, LoadImageW,
            MF_STRING, PostQuitMessage, RegisterClassW, SW_SHOWNORMAL, SetForegroundWindow,
            TPM_BOTTOMALIGN, TPM_LEFTALIGN, TPM_RIGHTBUTTON, TrackPopupMenu, TranslateMessage,
            WM_APP, WM_COMMAND, WM_DESTROY, WM_LBUTTONUP, WM_RBUTTONUP, WNDCLASSW,
        },
    },
};

const TRAY_CALLBACK: u32 = WM_APP + 1;
const OPEN_OBERIZ: usize = 1;
const EXIT_OBERIZ: usize = 2;
const START_SERVICE: usize = 3;
const STOP_SERVICE: usize = 4;
const CLOSE_SERVICE_AND_EXIT: usize = 5;
const WINDOW_CLASS: &str = "OberizTrayWindow";

fn notification_event(l_param: LPARAM) -> u32 {
    l_param as u32 & 0xffff
}

/// Set once before the message loop starts; `window_proc` is a plain Win32
/// callback with no way to capture state, so this is the simplest way to let
/// it know which of the two menus/behaviors is active. There is only ever
/// one tray per process.
static STANDALONE_MODE: AtomicBool = AtomicBool::new(false);

/// Embedded in the portable/default process: "Exit" quits this same process.
pub fn spawn() {
    std::thread::spawn(|| unsafe { tray_loop() });
}

/// A standalone helper process (`Oberiz.exe --tray`) that only shows the icon
/// and starts/stops the real Windows Service — it runs no web server itself.
pub fn run_standalone() {
    let Some(single_instance) = acquire_standalone_lock() else {
        return;
    };
    STANDALONE_MODE.store(true, Ordering::SeqCst);
    unsafe { tray_loop() };
    unsafe { CloseHandle(single_instance) };
}

/// There can be only one interactive tray helper per logged-in Windows user.
/// Both the Startup shortcut and the normal Oberiz shortcut may launch it,
/// so use a Local namespace mutex to discard duplicate helpers cleanly.
fn acquire_standalone_lock() -> Option<HANDLE> {
    let name = wide("Local\\OberizTraySingleton");
    let handle = unsafe { CreateMutexW(ptr::null(), 0, name.as_ptr()) };
    if handle.is_null() {
        return None;
    }
    if unsafe { GetLastError() } == ERROR_ALREADY_EXISTS {
        unsafe { CloseHandle(handle) };
        return None;
    }
    Some(handle)
}

unsafe fn tray_loop() {
    let class_name = wide(WINDOW_CLASS);
    let window_class = WNDCLASSW {
        lpfnWndProc: Some(window_proc),
        lpszClassName: class_name.as_ptr(),
        ..Default::default()
    };
    if RegisterClassW(&window_class) == 0 {
        return;
    }

    let window = CreateWindowExW(
        0,
        class_name.as_ptr(),
        class_name.as_ptr(),
        0,
        0,
        0,
        0,
        0,
        ptr::null_mut(),
        ptr::null_mut(),
        ptr::null_mut(),
        ptr::null(),
    );
    if window.is_null() {
        return;
    }

    let icon_path = std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(|directory| directory.join("oberiz.ico")));
    let icon = icon_path
        .as_deref()
        .map(|path| load_icon(path))
        .unwrap_or(ptr::null_mut());
    if icon.is_null() {
        return;
    }

    let mut notification = NOTIFYICONDATAW {
        cbSize: std::mem::size_of::<NOTIFYICONDATAW>() as u32,
        hWnd: window,
        uID: 1,
        uFlags: NIF_ICON | NIF_MESSAGE | NIF_TIP,
        uCallbackMessage: TRAY_CALLBACK,
        hIcon: icon,
        ..Default::default()
    };
    copy_wide(&mut notification.szTip, "Oberiz — Download client");
    if Shell_NotifyIconW(NIM_ADD, &notification) == 0 {
        return;
    }
    notification.Anonymous.uVersion = NOTIFYICON_VERSION_4;
    Shell_NotifyIconW(NIM_SETVERSION, &notification);

    let mut message = std::mem::zeroed();
    while GetMessageW(&mut message, ptr::null_mut(), 0, 0) > 0 {
        TranslateMessage(&message);
        DispatchMessageW(&message);
    }
    Shell_NotifyIconW(NIM_DELETE, &notification);
}

unsafe extern "system" fn window_proc(
    window: HWND,
    message: u32,
    w_param: WPARAM,
    l_param: LPARAM,
) -> LRESULT {
    match message {
        TRAY_CALLBACK
            if notification_event(l_param) == WM_LBUTTONUP
                || notification_event(l_param) == WM_RBUTTONUP =>
        {
            // With NOTIFYICON_VERSION_4 Windows stores the notification event
            // in the low word and the icon ID in the high word of lParam.
            // Comparing the complete value made right-clicks look unknown.
            if notification_event(l_param) == WM_LBUTTONUP {
                open_oberiz();
            } else {
                show_menu(window);
            }
            0
        }
        WM_COMMAND => match w_param & 0xffff {
            OPEN_OBERIZ => {
                open_oberiz();
                0
            }
            EXIT_OBERIZ => {
                PostQuitMessage(0);
                0
            }
            START_SERVICE => {
                control_service("start");
                0
            }
            STOP_SERVICE => {
                control_service("stop");
                0
            }
            CLOSE_SERVICE_AND_EXIT => {
                control_service("stop");
                PostQuitMessage(0);
                0
            }
            _ => DefWindowProcW(window, message, w_param, l_param),
        },
        WM_DESTROY => {
            PostQuitMessage(0);
            0
        }
        _ => DefWindowProcW(window, message, w_param, l_param),
    }
}

unsafe fn show_menu(window: HWND) {
    let menu = CreatePopupMenu();
    if menu.is_null() {
        return;
    }
    let open = wide("Open Oberiz");
    AppendMenuW(menu, MF_STRING, OPEN_OBERIZ, open.as_ptr());
    if STANDALONE_MODE.load(Ordering::SeqCst) {
        let start = wide("Start service");
        let stop = wide("Stop service");
        let close = wide("Close Oberiz");
        AppendMenuW(menu, MF_STRING, START_SERVICE, start.as_ptr());
        AppendMenuW(menu, MF_STRING, STOP_SERVICE, stop.as_ptr());
        AppendMenuW(menu, MF_STRING, CLOSE_SERVICE_AND_EXIT, close.as_ptr());
    } else {
        let exit = wide("Exit Oberiz");
        AppendMenuW(menu, MF_STRING, EXIT_OBERIZ, exit.as_ptr());
    }
    let mut point = std::mem::zeroed();
    GetCursorPos(&mut point);
    SetForegroundWindow(window);
    TrackPopupMenu(
        menu,
        TPM_BOTTOMALIGN | TPM_LEFTALIGN | TPM_RIGHTBUTTON,
        point.x,
        point.y,
        0,
        window,
        ptr::null(),
    );
    DestroyMenu(menu);
}

unsafe fn open_oberiz() {
    let operation = wide("open");
    let url = wide("http://127.0.0.1:2032");
    ShellExecuteW(
        ptr::null_mut(),
        operation.as_ptr(),
        url.as_ptr(),
        ptr::null(),
        ptr::null(),
        SW_SHOWNORMAL,
    );
}

/// Shells out to `sc.exe`, whose service security descriptor is set up at
/// install time (see `service::install`) to allow Authenticated Users to
/// start/stop the Oberiz service without an elevation prompt.
fn control_service(action: &str) {
    let _ = std::process::Command::new("sc")
        .args([action, crate::service::SERVICE_NAME])
        .output();
}

unsafe fn load_icon(path: &Path) -> *mut std::ffi::c_void {
    let path = path
        .as_os_str()
        .encode_wide()
        .chain(iter::once(0))
        .collect::<Vec<_>>();
    LoadImageW(
        ptr::null_mut(),
        path.as_ptr(),
        IMAGE_ICON,
        0,
        0,
        LR_LOADFROMFILE,
    )
}

fn wide(value: &str) -> Vec<u16> {
    value.encode_utf16().chain(iter::once(0)).collect()
}

fn copy_wide(destination: &mut [u16], value: &str) {
    for (slot, character) in destination.iter_mut().zip(value.encode_utf16()) {
        *slot = character;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tray_notification_event_ignores_the_icon_id_in_the_high_word() {
        let right_click_for_icon_one = ((1_u32 << 16) | WM_RBUTTONUP) as LPARAM;
        assert_eq!(notification_event(right_click_for_icon_one), WM_RBUTTONUP);
    }
}
