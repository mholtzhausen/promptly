//! Global hotkey registration: X11 XGrabKey with rdev fallback.

/// Register the global Ctrl+Alt+Space hotkey.
///
/// On X11: uses `XGrabKey` (no special permissions needed).
/// On Wayland/fallback: uses rdev evdev (requires `input` group membership).
pub fn register_global_hotkey(tx: std::sync::mpsc::Sender<()>) {
    #[cfg(target_os = "linux")]
    {
        if register_x11_grab(&tx) {
            return;
        }
        log::warn!("X11 hotkey grab failed, falling back to rdev evdev...");
    }

    register_rdev_hotkey(tx);
}

/// Register Ctrl+Alt+Space via X11 `XGrabKey`. Returns true on success.
#[cfg(target_os = "linux")]
fn register_x11_grab(tx: &std::sync::mpsc::Sender<()>) -> bool {
    use std::os::raw::{c_int, c_uint};
    use x11::xlib;

    unsafe {
        let display = xlib::XOpenDisplay(std::ptr::null());
        if display.is_null() {
            return false;
        }

        let root = xlib::XDefaultRootWindow(display);
        let keycode = xlib::XKeysymToKeycode(display, 0x0020);
        if keycode == 0 {
            xlib::XCloseDisplay(display);
            return false;
        }

        let base = (xlib::ControlMask | xlib::Mod1Mask) as c_uint;
        let modifier_combos = [
            base,
            base | xlib::Mod2Mask as c_uint,
            base | xlib::LockMask as c_uint,
            base | xlib::Mod2Mask as c_uint | xlib::LockMask as c_uint,
        ];

        for &mods in &modifier_combos {
            xlib::XGrabKey(
                display,
                keycode as c_int,
                mods,
                root,
                xlib::False,
                xlib::GrabModeAsync,
                xlib::GrabModeAsync,
            );
        }

        xlib::XFlush(display);

        log::info!("Registered X11 global hotkey Ctrl+Alt+Space (XGrabKey)");

        let display_ptr = display as usize;
        let keycode_val = keycode as usize;
        let base_mods = xlib::ControlMask | xlib::Mod1Mask;
        let tx_clone = tx.clone();

        let thread_body: Box<dyn FnOnce() + Send> = Box::new(move || {
            let display = display_ptr as *mut xlib::Display;
            let want_keycode = keycode_val as c_uint;
            loop {
                let mut event = std::mem::zeroed::<xlib::XEvent>();
                xlib::XNextEvent(display, &mut event);
                let event_type = event.type_;
                if event_type == xlib::KeyPress as c_int {
                    let state = event.key.state;
                    let got_keycode = event.key.keycode;
                    if got_keycode == want_keycode && (state & base_mods) == base_mods {
                        let _ = tx_clone.send(());
                    }
                }
            }
        });

        std::thread::spawn(thread_body);

        true
    }
}

/// Fallback hotkey registration using rdev evdev (needs `input` group on Linux).
fn register_rdev_hotkey(tx: std::sync::mpsc::Sender<()>) {
    use rdev::{EventType, Key};
    use std::sync::{Arc, Mutex};

    let ctrl_pressed = Arc::new(Mutex::new(false));
    let alt_pressed = Arc::new(Mutex::new(false));
    let cp = Arc::clone(&ctrl_pressed);
    let ap = Arc::clone(&alt_pressed);

    log::info!("Registering global hotkey Ctrl+Alt+Space via rdev...");

    std::thread::spawn(move || {
        let result = rdev::listen(move |event| {
            if let EventType::KeyPress(key) = event.event_type {
                match key {
                    Key::ControlLeft | Key::ControlRight => *cp.lock().unwrap() = true,
                    Key::Alt | Key::AltGr => *ap.lock().unwrap() = true,
                    Key::Space if *cp.lock().unwrap() && *ap.lock().unwrap() => {
                        let _ = tx.send(());
                    }
                    _ => {}
                }
            } else if let EventType::KeyRelease(key) = event.event_type {
                match key {
                    Key::ControlLeft | Key::ControlRight => *cp.lock().unwrap() = false,
                    Key::Alt | Key::AltGr => *ap.lock().unwrap() = false,
                    _ => {}
                }
            }
        });

        if let Err(e) = result {
            log::warn!(
                "rdev listener error: {:?}. App will still work via tray menu. \
                 Try adding your user to the `input` group: sudo usermod -aG input $USER",
                e
            );
        }
    });
}
