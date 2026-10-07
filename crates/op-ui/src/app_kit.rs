//! Small AppKit calls that winit and muda don't offer (macOS only).

use objc2::msg_send;
use objc2::runtime::{AnyClass, AnyObject};

/// Hides the application, like the standard "Hide" menu item (which
/// OpenPhoto replaces to give it Photoshop's Ctrl+Cmd+H).
pub fn hide_app() {
    let Some(class) = AnyClass::get(c"NSApplication") else {
        return;
    };
    // SAFETY: called on the main thread from the event loop
    unsafe {
        let app: *mut AnyObject = msg_send![class, sharedApplication];
        if !app.is_null() {
            let _: () = msg_send![app, hide: std::ptr::null::<AnyObject>()];
        }
    }
}

/// Whether the event being handled is a key press: a menu item chosen
/// through its key equivalent rather than with the mouse.
pub fn handling_key_press() -> bool {
    /// `NSEventTypeKeyDown`
    const KEY_DOWN: usize = 10;
    let Some(class) = AnyClass::get(c"NSApplication") else {
        return false;
    };
    // SAFETY: menu actions run on the main thread
    unsafe {
        let app: *mut AnyObject = msg_send![class, sharedApplication];
        if app.is_null() {
            return false;
        }
        let event: *mut AnyObject = msg_send![app, currentEvent];
        if event.is_null() {
            return false;
        }
        let kind: usize = msg_send![event, type];
        kind == KEY_DOWN
    }
}

/// The main display's pixel density in pixels per inch: its native pixel
/// width over its physical width (Photoshop's View › Actual Size uses the
/// same; 255 on a 14-inch MacBook Pro). `None` when the display doesn't
/// report a size.
pub fn screen_ppi() -> Option<f32> {
    #[repr(C)]
    struct CGSize {
        width: f64,
        height: f64,
    }
    #[link(name = "CoreGraphics", kind = "framework")]
    unsafe extern "C" {
        fn CGMainDisplayID() -> u32;
        fn CGDisplayScreenSize(display: u32) -> CGSize;
        fn CGDisplayCopyDisplayMode(display: u32) -> *mut std::ffi::c_void;
        fn CGDisplayModeGetPixelWidth(mode: *mut std::ffi::c_void) -> usize;
        fn CGDisplayModeRelease(mode: *mut std::ffi::c_void);
    }
    // SAFETY: plain CoreGraphics queries; the copied mode is released
    unsafe {
        let display = CGMainDisplayID();
        let mm = CGDisplayScreenSize(display).width;
        let mode = CGDisplayCopyDisplayMode(display);
        if mode.is_null() || mm <= 0.0 {
            return None;
        }
        let pixels = CGDisplayModeGetPixelWidth(mode);
        CGDisplayModeRelease(mode);
        Some((pixels as f64 / (mm / 25.4)) as f32)
    }
}

/// Whether Option is held right now (`NSEvent.modifierFlags`), for menu
/// commands that change with it.
pub fn option_held() -> bool {
    /// `NSEventModifierFlagOption`
    const OPTION: usize = 1 << 19;
    let Some(class) = AnyClass::get(c"NSEvent") else {
        return false;
    };
    // SAFETY: a class method reading the current modifier keys
    let flags: usize = unsafe { msg_send![class, modifierFlags] };
    flags & OPTION != 0
}
