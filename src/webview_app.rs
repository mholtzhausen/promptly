//! Tao/Wry application shell: owns the native window, webview, IPC backend,
//! and tray handle, and drives the event loop.

use std::cell::{Cell, RefCell};
use std::path::PathBuf;

use crate::config::AppConfig;

use tao::dpi::{LogicalSize, PhysicalPosition, PhysicalSize};
use tao::event::{Event, StartCause, WindowEvent};
use tao::event_loop::{ControlFlow, EventLoop, EventLoopProxy};
use tao::window::WindowBuilder;
use wry::WebViewBuilder;

use crate::ipc::{IpcBackend, RealDesktopEffects};
use crate::tray::TrayState;
use crate::window_focus;

/// User events delivered to the Tao event loop.
#[derive(Debug, Clone)]
pub enum AppEvent {
    /// Toggle window visibility (hotkey / tray).
    ToggleWindow,
    /// Raw IPC JSON string from the webview.
    Ipc(String),
}

/// The Promptly webview application: window + webview + backend + tray.
pub struct PromptlyWebviewApp {
    window: tao::window::Window,
    webview: wry::WebView,
    backend: IpcBackend,
    _tray: TrayState,
    /// True while waiting for the window to gain focus after show.
    focus_pending: Cell<bool>,
    /// True until the frontend on-show hook has run for this show cycle.
    on_show_pending: Cell<bool>,
    app_config: RefCell<AppConfig>,
}

impl PromptlyWebviewApp {
    /// Build the (initially hidden) window and webview.
    pub fn new(
        event_loop: &EventLoop<AppEvent>,
        proxy: EventLoopProxy<AppEvent>,
        tray: TrayState,
        db_path: PathBuf,
    ) -> anyhow::Result<Self> {
        let app_config = AppConfig::load();
        let window_size = app_config.window_size();
        let initial_size = LogicalSize::new(window_size.width, window_size.height);

        // Build window. On Linux, prevent Tao's default GtkBox so Wry's
        // build_gtk can add the WebKitWebView directly.
        #[cfg(target_os = "linux")]
        let window = {
            use tao::platform::unix::WindowBuilderExtUnix;
            WindowBuilder::new()
                .with_title("Promptly | Find a prompt")
                .with_inner_size(initial_size)
                .with_visible(false)
                .with_decorations(true)
                .with_always_on_top(true)
                .with_default_vbox(false)
                .build(event_loop)?
        };
        #[cfg(not(target_os = "linux"))]
        let window = WindowBuilder::new()
            .with_title("Promptly | Find a prompt")
            .with_inner_size(initial_size)
            .with_visible(false)
            .with_decorations(true)
            .with_always_on_top(true)
            .build(event_loop)?;

        let proxy_for_ipc = proxy.clone();
        let builder = WebViewBuilder::new()
            .with_html(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/frontend/dist/index.html")))
            .with_ipc_handler(move |request| {
                let _ = proxy_for_ipc.send_event(AppEvent::Ipc(request.body().clone()));
            })
            .with_navigation_handler(|url| url == "about:blank" || url.starts_with("data:"))
            .with_new_window_req_handler(|_, _| wry::NewWindowResponse::Deny);

        #[cfg(target_os = "linux")]
        let webview = {
            use tao::platform::unix::WindowExtUnix;
            use wry::WebViewBuilderExtUnix;
            builder.build_gtk(window.gtk_window())?
        };
        #[cfg(not(target_os = "linux"))]
        let webview = builder.build(&window)?;

        Ok(Self {
            window,
            webview,
            backend: IpcBackend::new(db_path),
            _tray: tray,
            focus_pending: Cell::new(false),
            on_show_pending: Cell::new(false),
            app_config: RefCell::new(app_config),
        })
    }

    /// Run the event loop until the process exits.
    pub fn run(
        event_loop: EventLoop<AppEvent>,
        state: PromptlyWebviewApp,
        show_on_start: bool,
    ) -> ! {
        let mut show_on_start = show_on_start;
        event_loop.run(move |event, _target, control_flow| {
            *control_flow = ControlFlow::Wait;
            if show_on_start {
                if matches!(&event, Event::NewEvents(StartCause::Init)) {
                    show_on_start = false;
                    state.show_window();
                }
            }
            match event {
                Event::UserEvent(AppEvent::ToggleWindow) => state.toggle_window(),
                Event::UserEvent(AppEvent::Ipc(raw)) => state.handle_ipc(&raw),
                Event::WindowEvent {
                    event: WindowEvent::Focused(true),
                    window_id,
                    ..
                } if window_id == state.window.id() => state.on_window_focused(),
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    ..
                } => state.window.set_visible(false),
                Event::WindowEvent {
                    event: WindowEvent::Resized(_),
                    window_id,
                    ..
                } if window_id == state.window.id() => state.on_window_resized(),
                _ => {}
            }
        });
    }

    fn toggle_window(&self) {
        if self.window.is_visible() {
            self.window.set_visible(false);
        } else {
            self.show_window();
        }
    }

    fn show_window(&self) {
        let window_size = self.app_config.borrow().window_size();
        self.window
            .set_inner_size(LogicalSize::new(window_size.width, window_size.height));

        if let Some(monitor) = self
            .window
            .current_monitor()
            .or_else(|| self.window.primary_monitor())
        {
            let scale = self.window.scale_factor();
            let win_w = window_size.width * scale;
            let win_h = window_size.height * scale;
            let PhysicalPosition { x: mx, y: my } = monitor.position();
            let PhysicalSize { width: mw, height: mh } = monitor.size();
            let cx = mx + ((mw as f64 - win_w) / 2.0).round() as i32;
            let cy = my + ((mh as f64 - win_h) / 2.0).round() as i32;
            self.window
                .set_outer_position(PhysicalPosition::new(cx, cy));
        }

        self.focus_pending.set(true);
        self.on_show_pending.set(true);
        self.window.set_always_on_top(true);
        self.window.set_visible(true);
        window_focus::present_and_activate(&self.window);
        self.focus_webview();
        self.finalize_show();
    }

    fn on_window_resized(&self) {
        let physical = self.window.inner_size();
        let scale = self.window.scale_factor();
        let width = physical.width as f64 / scale;
        let height = physical.height as f64 / scale;
        let mut config = self.app_config.borrow_mut();
        config.set_window_size(width, height);
        if let Err(e) = config.save() {
            log::warn!("Failed to save window size to config: {e}");
        }
    }

    fn on_window_focused(&self) {
        if self.focus_pending.take() {
            self.focus_webview();
        }
        self.finalize_show();
    }

    fn finalize_show(&self) {
        if !self.on_show_pending.take() {
            return;
        }
        self.notify_frontend_show();
        // WM activation from a global hotkey can complete after the first focus attempt.
        let _ = self.webview.evaluate_script(
            "setTimeout(function(){ \
               window.__promptlyFocusSearch && window.__promptlyFocusSearch(); \
             }, 0); \
             setTimeout(function(){ \
               window.__promptlyFocusSearch && window.__promptlyFocusSearch(); \
             }, 75);",
        );
    }

    fn focus_webview(&self) {
        let _ = self.webview.focus_parent();
        let _ = self.webview.focus();
        self.window.set_focus();
    }

    fn notify_frontend_show(&self) {
        let _ = self.webview.evaluate_script(
            "window.__promptlyOnShow && window.__promptlyOnShow(); \
             window.__promptlyFocusSearch && window.__promptlyFocusSearch();",
        );
    }

    fn handle_ipc(&self, raw: &str) {
        let handled = self.backend.handle(raw, &RealDesktopEffects);
        let _ = self.webview.evaluate_script(&format!(
            "window.__promptlyReceive({});",
            handled.response_json
        ));
        if let Some(title) = handled.window_title {
            self.window.set_title(&title);
        }
        if handled.hide_window {
            self.window.set_visible(false);
        }
        if handled.quit_app {
            std::process::exit(0);
        }
    }
}
