//! Promptly — system-tray app for managing prompt templates.
//!
//! Global shortcut: Ctrl+Alt+Space toggles the prompt-manager webview window.
//! Variables syntax: {{name|type|default|description}}

mod config;
mod db;
mod ipc;
mod prompt_parser;
mod tray;
mod webview_app;
mod window_focus;

use anyhow::Result;
use std::sync::{Arc, Mutex};

struct CliOptions {
    show_on_start: bool,
}

fn parse_cli() -> CliOptions {
    let mut show_on_start = false;
    for arg in std::env::args().skip(1) {
        match arg.as_str() {
            "--show" | "-s" => show_on_start = true,
            _ => {}
        }
    }
    CliOptions { show_on_start }
}

/// Detach from the terminal by re-execing with `PROMPTLY_FOREGROUND=1`.
/// The parent exits immediately; the child keeps running in the background.
fn daemonize() {
    if std::env::var("PROMPTLY_FOREGROUND").is_ok() {
        return;
    }
    let exe = std::env::current_exe().expect("failed to get current exe path");
    let _child = std::process::Command::new(&exe)
        .env("PROMPTLY_FOREGROUND", "1")
        .args(std::env::args().skip(1))
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .expect("failed to daemonize");
    // Parent exits immediately; child lives on.
    std::process::exit(0);
}

fn main() -> Result<()> {
    let cli = parse_cli();
    daemonize();
    env_logger::init();

    // ── Ensure config directory exists ───────────────────────────────
    config::ensure_config_dir()?;

    // ── Tao/Wry event loop (replaces the GTK Application + polling) ──
    let event_loop =
        tao::event_loop::EventLoopBuilder::<webview_app::AppEvent>::with_user_event().build();
    let proxy = event_loop.create_proxy();

    // Channel for hotkey + tray triggers (both send ()).
    let (toggle_tx, toggle_rx) = std::sync::mpsc::channel::<()>();

    // ── System tray icon (ksni, GTK-independent) ────────────────────
    let tray_state = tray::TrayState::build(toggle_tx.clone())?;

    // ── Global hotkey registration via X11 (XGrabKey) or rdev ────────
    register_global_hotkey(toggle_tx.clone());

    // Forward () triggers into the Tao event loop as ToggleWindow events.
    let toggle_proxy = proxy.clone();
    std::thread::spawn(move || {
        while toggle_rx.recv().is_ok() {
            let _ = toggle_proxy.send_event(webview_app::AppEvent::ToggleWindow);
        }
    });

    // ── Build and run the webview application ─────────────────────────
    let app = webview_app::PromptlyWebviewApp::new(
        &event_loop,
        proxy.clone(),
        tray_state,
        config::db_path(),
    )?;
    webview_app::PromptlyWebviewApp::run(event_loop, app, cli.show_on_start);
}

/// Register the global Ctrl+Alt+Space hotkey.
///
/// On X11: uses `XGrabKey` (no special permissions needed).
/// On Wayland/fallback: uses rdev evdev (requires `input` group membership).
fn register_global_hotkey(tx: std::sync::mpsc::Sender<()>) {
    // Try X11 XGrabKey first — works on X11 without `input` group permissions.
    #[cfg(target_os = "linux")]
    {
        if register_x11_grab(&tx) {
            return;
        }
        log::warn!("X11 hotkey grab failed, falling back to rdev evdev...");
    }

    // Fallback: rdev (works on Wayland and when input group is available)
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
        // XK_space = 0x0020 (Latin 1)
        let keycode = xlib::XKeysymToKeycode(display, 0x0020);
        if keycode == 0 {
            xlib::XCloseDisplay(display);
            return false;
        }

        // Base modifiers for Ctrl+Alt
        let base = (xlib::ControlMask | xlib::Mod1Mask) as c_uint;
        // Also grab with NumLock (Mod2Mask) and/or CapsLock (LockMask) held,
        // so the hotkey works regardless of lock state.
        let modifier_combos = [
            base,
            base | xlib::Mod2Mask as c_uint,    // + NumLock
            base | xlib::LockMask as c_uint,     // + CapsLock
            base | xlib::Mod2Mask as c_uint | xlib::LockMask as c_uint, // + both
        ];

        for &mods in &modifier_combos {
            xlib::XGrabKey(
                display,
                keycode as c_int,
                mods,
                root,
                xlib::False,                 // owner_events
                xlib::GrabModeAsync,          // pointer_mode
                xlib::GrabModeAsync,          // keyboard_mode
            );
        }

        xlib::XFlush(display);

        log::info!("Registered X11 global hotkey Ctrl+Alt+Space (XGrabKey)");

        // Move the raw display pointer as a plain usize through the Send
        // boundary; X11 connections are not thread-safe by default, but this
        // listener thread becomes the sole user.
        let display_ptr = display as usize;
        let keycode_val = keycode as usize;
        // Modifier mask to check: any Control + any Alt is sufficient.
        let base_mods = (xlib::ControlMask | xlib::Mod1Mask) as u32;
        let tx_clone = tx.clone();

        let thread_body: Box<dyn FnOnce() + Send> = Box::new(move || {
            let display = display_ptr as *mut xlib::Display;
            let want_keycode = keycode_val as c_uint;
            loop {
                // XNextEvent blocks until an event arrives.
                let mut event = std::mem::zeroed::<xlib::XEvent>();
                xlib::XNextEvent(display, &mut event);
                let event_type = event.type_;
                if event_type == xlib::KeyPress as c_int {
                    let state = event.key.state;
                    let got_keycode = event.key.keycode;
                    if got_keycode == want_keycode
                        && (state & base_mods) == base_mods
                    {
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

    // Track modifier state for Ctrl+Alt detection
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
                    Key::Space => {
                        if *cp.lock().unwrap() && *ap.lock().unwrap() {
                            let _ = tx.send(());
                        }
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
