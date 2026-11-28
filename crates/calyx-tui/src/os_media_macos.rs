//! macOS only: Now Playing and the media keys reach an app through its
//! AppKit event loop, and calyx runs in a terminal with none: macOS then
//! ignores it (Control Center stays empty, play/pause opens Music). calyx
//! becomes a faceless NSApplication (no Dock icon, no window) and its events
//! are pumped from calyx's own loop, like the hidden window on Windows.

use objc::runtime::{BOOL, Object, YES};
use objc::{class, msg_send, sel, sel_impl};

type Id = *mut Object;

/// `NSApplicationActivationPolicyAccessory`: no Dock icon, no menu bar.
const ACCESSORY: i64 = 1;
/// `NSEventMaskAny`.
const ANY_EVENT: u64 = u64::MAX;

#[link(name = "AppKit", kind = "framework")]
unsafe extern "C" {
    static NSDefaultRunLoopMode: Id;
}

/// Turns this process into a faceless AppKit app (once; main thread only).
pub fn start() {
    static ONCE: std::sync::Once = std::sync::Once::new();
    // SAFETY: plain AppKit messages on the main thread, where calyx's event
    // loop runs; `app` is the process-wide shared application.
    ONCE.call_once(|| unsafe {
        let app: Id = msg_send![class!(NSApplication), sharedApplication];
        let _: BOOL = msg_send![app, setActivationPolicy: ACCESSORY];
        let _: () = msg_send![app, finishLaunching];
    });
}

/// Handles every AppKit event waiting, without blocking (main thread only).
pub fn pump() {
    // SAFETY: as in `start`; the pool releases the autoreleased events.
    unsafe {
        let pool: Id = msg_send![class!(NSAutoreleasePool), new];
        let app: Id = msg_send![class!(NSApplication), sharedApplication];
        let past: Id = msg_send![class!(NSDate), distantPast];
        loop {
            let event: Id = msg_send![app, nextEventMatchingMask: ANY_EVENT
                                                untilDate: past
                                                   inMode: NSDefaultRunLoopMode
                                                  dequeue: YES];
            if event.is_null() {
                break;
            }
            let _: () = msg_send![app, sendEvent: event];
        }
        let _: () = msg_send![pool, drain];
    }
}
