//! Window-manager hints shared by popup-style windows.

use gtk4::glib;
use gtk4::glib::translate::ToGlibPtr;
use gtk4::prelude::*;
use gtk4::{gdk, Widget, Window};

#[cfg(target_os = "linux")]
extern "C" {
    fn gdk_x11_surface_get_xid(surface: *mut gdk::ffi::GdkSurface) -> std::os::raw::c_ulong;
}

/// Present a GTK window, center it on the current monitor, keep it above other windows, and focus
/// the requested child widget once the native surface exists.
pub fn present_centered_always_on_top<W, F>(
    window: &W,
    fallback_width: i32,
    fallback_height: i32,
    focus_widget: Option<&F>,
) where
    W: IsA<Window>,
    F: IsA<Widget>,
{
    let window = window.as_ref();
    let focus_widget = focus_widget.map(|widget| widget.as_ref().clone());

    window.present();
    focus_child(window, focus_widget.as_ref());

    let window = window.clone();
    glib::idle_add_local_once(move || {
        apply_now(&window, fallback_width, fallback_height);
        window.present();
        focus_child(&window, focus_widget.as_ref());
    });
}

/// Apply native positioning/stacking hints immediately. Safe no-op where the backend does not expose
/// the controls GTK needs for regular toplevel windows.
pub fn apply_now<W: IsA<Window>>(window: &W, fallback_width: i32, fallback_height: i32) {
    let window = window.as_ref();
    center_window_on_screen(window, fallback_width, fallback_height);
    keep_window_above(window);
}

fn focus_child(window: &Window, focus_widget: Option<&Widget>) {
    if let Some(focus_widget) = focus_widget {
        gtk4::prelude::GtkWindowExt::set_focus(window, Some(focus_widget));
        focus_widget.grab_focus();
    }
}

#[cfg(target_os = "linux")]
fn x11_window_id(window: &Window) -> Option<std::os::raw::c_ulong> {
    let display = gtk4::prelude::WidgetExt::display(window);
    if display.type_().name() != "GdkX11Display" {
        return None;
    }

    let native = window.native()?;
    let surface = native.surface()?;
    let xid = unsafe { gdk_x11_surface_get_xid(surface.to_glib_none().0) };
    if xid == 0 {
        None
    } else {
        Some(xid)
    }
}

#[cfg(target_os = "linux")]
fn center_window_on_screen(window: &Window, fallback_width: i32, fallback_height: i32) {
    let Some(xid) = x11_window_id(window) else {
        return;
    };
    let Some(native) = window.native() else {
        return;
    };
    let Some(surface) = native.surface() else {
        return;
    };
    let display = gtk4::prelude::WidgetExt::display(window);
    let monitor = display
        .monitor_at_surface(&surface)
        .or_else(|| display.monitors().item(0).and_downcast::<gdk::Monitor>());
    let Some(monitor) = monitor else {
        return;
    };

    let geometry = monitor.geometry();
    let width = window.width().max(fallback_width);
    let height = window.height().max(fallback_height);
    let x = geometry.x() + (geometry.width() - width).max(0) / 2;
    let y = geometry.y() + (geometry.height() - height).max(0) / 2;

    unsafe {
        let display = x11::xlib::XOpenDisplay(std::ptr::null());
        if display.is_null() {
            return;
        }
        x11::xlib::XMoveWindow(display, xid, x, y);
        x11::xlib::XFlush(display);
        x11::xlib::XCloseDisplay(display);
    }
}

#[cfg(not(target_os = "linux"))]
fn center_window_on_screen(_: &Window, _: i32, _: i32) {}

#[cfg(target_os = "linux")]
fn keep_window_above(window: &Window) {
    use std::os::raw::c_long;

    const NET_WM_STATE: &[u8] = b"_NET_WM_STATE\0";
    const NET_WM_STATE_ABOVE: &[u8] = b"_NET_WM_STATE_ABOVE\0";
    const NET_WM_STATE_ADD: c_long = 1;
    const SOURCE_NORMAL_APPLICATION: c_long = 1;

    let Some(xid) = x11_window_id(window) else {
        return;
    };

    unsafe {
        let display = x11::xlib::XOpenDisplay(std::ptr::null());
        if display.is_null() {
            return;
        }

        let wm_state =
            x11::xlib::XInternAtom(display, NET_WM_STATE.as_ptr().cast(), x11::xlib::False);
        let wm_state_above = x11::xlib::XInternAtom(
            display,
            NET_WM_STATE_ABOVE.as_ptr().cast(),
            x11::xlib::False,
        );
        if wm_state == 0 || wm_state_above == 0 {
            x11::xlib::XCloseDisplay(display);
            return;
        }

        x11::xlib::XChangeProperty(
            display,
            xid,
            wm_state,
            x11::xlib::XA_ATOM,
            32,
            x11::xlib::PropModeReplace,
            (&wm_state_above as *const x11::xlib::Atom).cast(),
            1,
        );

        let screen = x11::xlib::XDefaultScreen(display);
        let root = x11::xlib::XRootWindow(display, screen);
        let mut event = x11::xlib::XEvent::from(x11::xlib::XClientMessageEvent {
            type_: x11::xlib::ClientMessage,
            serial: 0,
            send_event: x11::xlib::True,
            display,
            window: xid,
            message_type: wm_state,
            format: 32,
            data: [
                NET_WM_STATE_ADD,
                wm_state_above as c_long,
                0,
                SOURCE_NORMAL_APPLICATION,
                0,
            ]
            .into(),
        });
        x11::xlib::XSendEvent(
            display,
            root,
            x11::xlib::False,
            x11::xlib::SubstructureRedirectMask | x11::xlib::SubstructureNotifyMask,
            &mut event,
        );
        x11::xlib::XRaiseWindow(display, xid);
        x11::xlib::XFlush(display);
        x11::xlib::XCloseDisplay(display);
    }
}

#[cfg(not(target_os = "linux"))]
fn keep_window_above(_: &Window) {}
