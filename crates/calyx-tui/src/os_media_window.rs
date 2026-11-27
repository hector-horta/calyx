//! Windows only: an invisible window for the media controls. SMTC attaches
//! to a window (`GetForWindow`) and calyx runs in a console, so it gets one
//! of its own, never shown. Its messages are pumped from the event loop.

use std::ffi::c_void;
use windows_sys::Win32::Foundation::HWND;
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
use windows_sys::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW, MSG, PM_REMOVE, PeekMessageW,
    RegisterClassW, TranslateMessage, WNDCLASSW, WS_OVERLAPPEDWINDOW,
};

/// Window class and title, as UTF-16 with the trailing NUL.
const CLASS_NAME: &[u16] = &[
    b'c' as u16,
    b'a' as u16,
    b'l' as u16,
    b'y' as u16,
    b'x' as u16,
    0,
];

pub struct HiddenWindow {
    hwnd: HWND,
}

impl HiddenWindow {
    /// Creates the window (never shown); `None` when Windows refuses.
    pub fn create() -> Option<Self> {
        // SAFETY: plain Win32 calls with valid, NUL-terminated strings that
        // live for the whole program (`CLASS_NAME` is a static).
        let instance = unsafe { GetModuleHandleW(std::ptr::null()) };
        let class = WNDCLASSW {
            lpfnWndProc: Some(DefWindowProcW),
            hInstance: instance,
            lpszClassName: CLASS_NAME.as_ptr(),
            // SAFETY: every other field of WNDCLASSW may be zero/null.
            ..unsafe { std::mem::zeroed() }
        };
        // Fails harmlessly when the class is already registered.
        // SAFETY: `class` is fully initialised and outlives the call.
        unsafe { RegisterClassW(&class) };
        // SAFETY: as above; no parent, menu or creation data.
        let hwnd = unsafe {
            CreateWindowExW(
                0,
                CLASS_NAME.as_ptr(),
                CLASS_NAME.as_ptr(),
                WS_OVERLAPPEDWINDOW,
                0,
                0,
                0,
                0,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                instance,
                std::ptr::null(),
            )
        };
        if hwnd.is_null() {
            None
        } else {
            Some(Self { hwnd })
        }
    }

    /// Returns the underlying HWND wrapped in a raw pointer.
    pub fn handle(&self) -> *mut c_void {
        self.hwnd
    }

    /// Pumps all messages for this hidden window without blocking.
    pub fn pump(&self) {
        // SAFETY: `msg` is a valid zeroed buffer, and `hwnd` is our live window.
        let mut msg: MSG = unsafe { std::mem::zeroed() };
        while unsafe { PeekMessageW(&mut msg, self.hwnd, 0, 0, PM_REMOVE) } != 0 {
            // SAFETY: `msg` contains valid message data and is not modified after this call.
            unsafe {
                TranslateMessage(&msg);
            }
            // SAFETY: `msg` is still valid for DispatchMessageW.
            unsafe {
                DispatchMessageW(&msg);
            }
        }
    }
}

impl Drop for HiddenWindow {
    fn drop(&mut self) {
        // SAFETY: `hwnd` was created by `create` and is destroyed only here.
        unsafe { DestroyWindow(self.hwnd) };
    }
}
