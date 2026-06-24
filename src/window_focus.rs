//! Native window activation for the launcher popup (Linux/X11).

#[cfg(target_os = "linux")]
pub fn present_and_activate(tao_window: &tao::window::Window) {
    use gtk::prelude::*;
    use tao::platform::unix::WindowExtUnix;

    let gtk_win = tao_window.gtk_window();
    gtk_win.present();
    x11_raise_and_keep_above(gtk_win);
    x11_request_active(gtk_win);

    let gtk_win = gtk_win.clone();
    glib::idle_add_local_once(move || {
        gtk_win.present_with_time(gdk::ffi::GDK_CURRENT_TIME as _);
    });
}

#[cfg(not(target_os = "linux"))]
pub fn present_and_activate(tao_window: &tao::window::Window) {
    tao_window.set_focus();
}

#[cfg(target_os = "linux")]
fn x11_window_id(gtk_win: &gtk::ApplicationWindow) -> Option<std::os::raw::c_ulong> {
    use gtk::prelude::*;
    let gdk_win = gtk_win.window()?;
    let x11 = gdk_win.downcast_ref::<gdkx11::X11Window>()?;
    Some(x11.xid() as std::os::raw::c_ulong)
}

#[cfg(target_os = "linux")]
fn x11_raise_and_keep_above(gtk_win: &gtk::ApplicationWindow) {
    use std::os::raw::c_long;

    const NET_WM_STATE: &[u8] = b"_NET_WM_STATE\0";
    const NET_WM_STATE_ABOVE: &[u8] = b"_NET_WM_STATE_ABOVE\0";
    const NET_WM_STATE_ADD: c_long = 1;
    const SOURCE_NORMAL_APPLICATION: c_long = 1;

    let Some(xid) = x11_window_id(gtk_win) else {
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

#[cfg(target_os = "linux")]
fn x11_request_active(gtk_win: &gtk::ApplicationWindow) {
    const NET_ACTIVE_WINDOW: &[u8] = b"_NET_ACTIVE_WINDOW\0";

    let Some(xid) = x11_window_id(gtk_win) else {
        return;
    };

    unsafe {
        let display = x11::xlib::XOpenDisplay(std::ptr::null());
        if display.is_null() {
            return;
        }

        let active = x11::xlib::XInternAtom(
            display,
            NET_ACTIVE_WINDOW.as_ptr().cast(),
            x11::xlib::False,
        );
        if active == 0 {
            x11::xlib::XCloseDisplay(display);
            return;
        }

        let screen = x11::xlib::XDefaultScreen(display);
        let root = x11::xlib::XRootWindow(display, screen);
        let mut event = x11::xlib::XEvent::from(x11::xlib::XClientMessageEvent {
            type_: x11::xlib::ClientMessage,
            serial: 0,
            send_event: x11::xlib::True,
            display,
            window: xid,
            message_type: active,
            format: 32,
            data: [1_i64, 0, 0, 0, 0].into(),
        });
        x11::xlib::XSendEvent(
            display,
            root,
            x11::xlib::False,
            x11::xlib::SubstructureRedirectMask | x11::xlib::SubstructureNotifyMask,
            &mut event,
        );
        x11::xlib::XFlush(display);
        x11::xlib::XCloseDisplay(display);
    }
}
