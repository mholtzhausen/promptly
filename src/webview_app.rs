//! Tao/Wry application shell: owns the native window, webview, IPC backend,
//! and tray handle, and drives the event loop.

use std::cell::{Cell, RefCell};
use std::path::PathBuf;
use std::rc::Rc;

use crate::config::{AppConfig, WindowSize, ABOUT_WINDOW_HEIGHT, ABOUT_WINDOW_WIDTH};

use tao::dpi::{LogicalSize, PhysicalPosition, PhysicalSize};
use tao::event::{Event, StartCause, WindowEvent};
use tao::event_loop::{ControlFlow, EventLoop, EventLoopProxy, EventLoopWindowTarget};
use tao::window::{Window, WindowBuilder};
use wry::WebViewBuilder;

use crate::ipc::{AppNotification, DesktopEffects, IpcBackend, NotificationCollector};
use crate::tray::TrayState;
use crate::update::UpdateCheckOutcome;
use crate::window_focus;

/// Which webview originated an IPC message.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IpcSource {
    Main,
    Settings,
}

/// User events delivered to the Tao event loop.
#[derive(Debug, Clone)]
pub enum AppEvent {
    /// Toggle window visibility (hotkey / tray).
    ToggleWindow,
    /// Raw IPC JSON string from a webview.
    Ipc { raw: String, source: IpcSource },
    /// Map the window after geometry has been applied while still hidden.
    RevealWindow,
    /// Check GitHub for a newer release.
    CheckForUpdates { user_initiated: bool },
    /// Result of a background update check.
    UpdateCheckResult {
        outcome: Result<UpdateCheckOutcome, String>,
        user_initiated: bool,
    },
    /// Open the fixed-size About pane.
    ShowAbout,
    /// Push in-app notifications to the React footer.
    PushNotifications(Vec<AppNotification>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WindowPane {
    Main,
    About,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ShowIntent {
    Main,
    About,
}

struct SettingsWindowState {
    window: Window,
    webview: wry::WebView,
}

/// The Promptly webview application: window + webview + backend + tray.
pub struct PromptlyWebviewApp {
    window: Window,
    webview: wry::WebView,
    backend: IpcBackend,
    _tray: TrayState,
    event_proxy: EventLoopProxy<AppEvent>,
    /// True while waiting for the window to gain focus after show.
    focus_pending: Cell<bool>,
    /// True until the frontend on-show hook has run for this show cycle.
    on_show_pending: Cell<bool>,
    show_intent: Cell<Option<ShowIntent>>,
    window_pane: Cell<WindowPane>,
    app_config: Rc<RefCell<AppConfig>>,
    settings_window: RefCell<Option<SettingsWindowState>>,
    settings_focus_pending: Cell<bool>,
}

impl PromptlyWebviewApp {
    /// Build the (initially hidden) window and webview.
    pub fn new(
        event_loop: &EventLoop<AppEvent>,
        proxy: EventLoopProxy<AppEvent>,
        tray: TrayState,
        db_path: PathBuf,
    ) -> anyhow::Result<Self> {
        let app_config = Rc::new(RefCell::new(AppConfig::load()));
        let window_size = app_config.borrow().window_size();
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
            .with_html(include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/frontend/dist/index.html"
            )))
            .with_ipc_handler(move |request| {
                let _ = proxy_for_ipc.send_event(AppEvent::Ipc {
                    raw: request.body().clone(),
                    source: IpcSource::Main,
                });
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
            backend: IpcBackend::new(db_path, app_config.clone())?,
            _tray: tray,
            event_proxy: proxy,
            focus_pending: Cell::new(false),
            on_show_pending: Cell::new(false),
            show_intent: Cell::new(None),
            window_pane: Cell::new(WindowPane::Main),
            app_config,
            settings_window: RefCell::new(None),
            settings_focus_pending: Cell::new(false),
        })
    }

    /// Run the event loop until the process exits.
    pub fn run(
        event_loop: EventLoop<AppEvent>,
        state: PromptlyWebviewApp,
        show_on_start: bool,
    ) -> ! {
        let mut show_on_start = show_on_start;
        let mut startup_update_check = true;
        event_loop.run(move |event, target, control_flow| {
            *control_flow = ControlFlow::Wait;
            if show_on_start && matches!(&event, Event::NewEvents(StartCause::Init)) {
                show_on_start = false;
                state.show_window();
            }
            if startup_update_check && matches!(&event, Event::NewEvents(StartCause::Init)) {
                startup_update_check = false;
                state.spawn_update_check(false);
            }
            match event {
                Event::UserEvent(AppEvent::ToggleWindow) => state.toggle_window(),
                Event::UserEvent(AppEvent::RevealWindow) => state.reveal_window(),
                Event::UserEvent(AppEvent::Ipc { raw, source }) => {
                    state.handle_ipc(&raw, source, &target);
                }
                Event::UserEvent(AppEvent::CheckForUpdates { user_initiated }) => {
                    state.spawn_update_check(user_initiated);
                }
                Event::UserEvent(AppEvent::UpdateCheckResult {
                    outcome,
                    user_initiated,
                }) => state.handle_update_check_result(outcome, user_initiated),
                Event::UserEvent(AppEvent::ShowAbout) => state.show_about_pane(),
                Event::UserEvent(AppEvent::PushNotifications(notifications)) => {
                    state.push_notifications(&notifications);
                }
                Event::WindowEvent {
                    event: WindowEvent::Focused(true),
                    window_id,
                    ..
                } if window_id == state.window.id() => state.on_window_focused(),
                Event::WindowEvent {
                    event: WindowEvent::Focused(true),
                    window_id,
                    ..
                } if state.is_settings_window_id(window_id) => state.on_settings_focused(),
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    window_id,
                    ..
                } if window_id == state.window.id() => state.hide_window(),
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    window_id,
                    ..
                } if state.is_settings_window_id(window_id) => state.close_settings_window(),
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
            self.hide_window();
        } else {
            self.show_window();
        }
    }

    fn about_window_size() -> WindowSize {
        WindowSize {
            width: ABOUT_WINDOW_WIDTH,
            height: ABOUT_WINDOW_HEIGHT,
        }
    }

    fn pane_window_size(&self, pane: WindowPane) -> WindowSize {
        match pane {
            WindowPane::Main => self.app_config.borrow().window_size(),
            WindowPane::About => Self::about_window_size(),
        }
    }

    fn apply_geometry(&self, window_size: WindowSize) {
        self.window
            .set_inner_size(LogicalSize::new(window_size.width, window_size.height));
        if let Some(pos) = self.centered_position(window_size) {
            self.window.set_outer_position(pos);
            window_focus::x11_move_window(&self.window, pos);
        }
    }

    fn restore_main_geometry(&self) {
        if self.window_pane.get() == WindowPane::Main {
            return;
        }
        self.window_pane.set(WindowPane::Main);
        self.apply_geometry(self.app_config.borrow().window_size());
    }

    fn hide_window(&self) {
        window_focus::set_window_opacity(&self.window, 1.0);
        self.restore_main_geometry();
        self.window.set_visible(false);
        self.notify_frontend_window_hidden();
    }

    fn centered_position(
        &self,
        window_size: crate::config::WindowSize,
    ) -> Option<PhysicalPosition<i32>> {
        let monitor = self
            .window
            .current_monitor()
            .or_else(|| self.window.primary_monitor())?;
        let scale = self.window.scale_factor();
        let win_w = window_size.width * scale;
        let win_h = window_size.height * scale;
        let PhysicalPosition { x: mx, y: my } = monitor.position();
        let PhysicalSize {
            width: mw,
            height: mh,
        } = monitor.size();
        let cx = mx + ((mw as f64 - win_w) / 2.0).round() as i32;
        let cy = my + ((mh as f64 - win_h) / 2.0).round() as i32;
        Some(PhysicalPosition::new(cx, cy))
    }

    /// Position the hidden window, then defer mapping to the next event-loop tick.
    fn show_window(&self) {
        self.window_pane.set(WindowPane::Main);
        self.apply_geometry(self.pane_window_size(WindowPane::Main));
        self.focus_pending.set(true);
        self.on_show_pending.set(true);
        self.show_intent.set(Some(ShowIntent::Main));
        self.window.set_always_on_top(true);
        window_focus::set_window_opacity(&self.window, 0.0);
        let _ = self.event_proxy.send_event(AppEvent::RevealWindow);
    }

    fn show_about_pane(&self) {
        self.window_pane.set(WindowPane::About);
        self.apply_geometry(self.pane_window_size(WindowPane::About));
        self.focus_pending.set(true);
        self.on_show_pending.set(true);
        self.show_intent.set(Some(ShowIntent::About));
        self.window.set_always_on_top(true);
        window_focus::set_window_opacity(&self.window, 0.0);
        let _ = self.event_proxy.send_event(AppEvent::RevealWindow);
    }

    /// Map and activate the window after geometry was applied in `show_window`.
    fn reveal_window(&self) {
        self.window.set_visible(true);
        window_focus::present_and_activate(&self.window);
        self.schedule_reveal_opacity();
        self.finalize_show();
    }

    fn schedule_reveal_opacity(&self) {
        #[cfg(target_os = "linux")]
        {
            use gtk::prelude::*;
            use tao::platform::unix::WindowExtUnix;

            let gtk_win = self.window.gtk_window().clone();
            glib::idle_add_local_once(move || gtk_win.set_opacity(1.0));
        }
        #[cfg(not(target_os = "linux"))]
        {
            window_focus::set_window_opacity(&self.window, 1.0);
        }
    }

    fn on_window_resized(&self) {
        if self.window_pane.get() != WindowPane::Main {
            return;
        }
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
        match self.show_intent.take() {
            Some(ShowIntent::Main) => {
                self.notify_frontend_show();
                self.notify_frontend_window_visible();
            }
            Some(ShowIntent::About) => {
                self.notify_frontend_about();
                self.notify_frontend_window_visible();
            }
            None => {}
        }
    }

    fn focus_webview(&self) {
        let _ = self.webview.focus_parent();
        let _ = self.webview.focus();
        self.window.set_focus();
    }

    fn notify_frontend_show(&self) {
        let _ = self
            .webview
            .evaluate_script("window.__promptlyOnShow && window.__promptlyOnShow();");
    }

    fn notify_frontend_about(&self) {
        let _ = self
            .webview
            .evaluate_script("window.__promptlyShowAbout && window.__promptlyShowAbout();");
    }

    fn notify_frontend_window_visible(&self) {
        let _ = self.webview.evaluate_script(
            "window.__promptlyOnWindowVisible && window.__promptlyOnWindowVisible();",
        );
    }

    fn notify_frontend_window_hidden(&self) {
        let _ = self.webview.evaluate_script(
            "window.__promptlyOnWindowHidden && window.__promptlyOnWindowHidden();",
        );
    }

    fn ephemeral_notification_ms(&self) -> u64 {
        self.app_config.borrow().ephemeral_notification_ms()
    }

    fn is_settings_window_id(&self, window_id: tao::window::WindowId) -> bool {
        self.settings_window
            .borrow()
            .as_ref()
            .is_some_and(|s| s.window.id() == window_id)
    }

    fn center_on_window(
        parent: &Window,
        child_size: WindowSize,
    ) -> Option<PhysicalPosition<i32>> {
        let scale = parent.scale_factor();
        let parent_pos = parent.outer_position().ok()?;
        let parent_size = parent.outer_size();
        let child_w = (child_size.width * scale).round() as i32;
        let child_h = (child_size.height * scale).round() as i32;
        let x = parent_pos.x + ((parent_size.width as i32 - child_w) / 2);
        let y = parent_pos.y + ((parent_size.height as i32 - child_h) / 2);
        Some(PhysicalPosition::new(x, y))
    }

    fn open_settings_window(&self, target: &EventLoopWindowTarget<AppEvent>) {
        if let Some(settings) = self.settings_window.borrow().as_ref() {
            let size = self.app_config.borrow().settings_window_size();
            settings
                .window
                .set_inner_size(LogicalSize::new(size.width, size.height));
            if let Some(pos) = Self::center_on_window(&self.window, size) {
                settings.window.set_outer_position(pos);
                window_focus::x11_move_window(&settings.window, pos);
            }
            settings.window.set_always_on_top(true);
            settings.window.set_visible(true);
            window_focus::present_and_activate(&settings.window);
            self.settings_focus_pending.set(true);
            return;
        }

        let size = self.app_config.borrow().settings_window_size();
        let initial_size = LogicalSize::new(size.width, size.height);

        #[cfg(target_os = "linux")]
        let settings_win = {
            use tao::platform::unix::WindowBuilderExtUnix;
            WindowBuilder::new()
                .with_title("Promptly | Settings")
                .with_inner_size(initial_size)
                .with_resizable(false)
                .with_visible(true)
                .with_decorations(true)
                .with_always_on_top(true)
                .with_default_vbox(false)
                .build(target)
        };
        #[cfg(not(target_os = "linux"))]
        let settings_win = WindowBuilder::new()
            .with_title("Promptly | Settings")
            .with_inner_size(initial_size)
            .with_resizable(false)
            .with_visible(true)
            .with_decorations(true)
            .with_always_on_top(true)
            .build(target);

        let settings_win = match settings_win {
            Ok(w) => w,
            Err(e) => {
                log::error!("Failed to create settings window: {e}");
                return;
            }
        };

        if let Some(pos) = Self::center_on_window(&self.window, size) {
            settings_win.set_outer_position(pos);
            window_focus::x11_move_window(&settings_win, pos);
        }

        let proxy_for_ipc = self.event_proxy.clone();
        let builder = WebViewBuilder::new()
            .with_html(include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/frontend/dist/index.html"
            )))
            .with_ipc_handler(move |request| {
                let _ = proxy_for_ipc.send_event(AppEvent::Ipc {
                    raw: request.body().clone(),
                    source: IpcSource::Settings,
                });
            })
            .with_navigation_handler(|url| url == "about:blank" || url.starts_with("data:"))
            .with_new_window_req_handler(|_, _| wry::NewWindowResponse::Deny)
            .with_initialization_script("window.__promptlyWindowRole='settings';");

        #[cfg(target_os = "linux")]
        let settings_webview = {
            use tao::platform::unix::WindowExtUnix;
            use wry::WebViewBuilderExtUnix;
            match builder.build_gtk(settings_win.gtk_window()) {
                Ok(wv) => wv,
                Err(e) => {
                    log::error!("Failed to create settings webview: {e}");
                    return;
                }
            }
        };
        #[cfg(not(target_os = "linux"))]
        let settings_webview = match builder.build(&settings_win) {
            Ok(wv) => wv,
            Err(e) => {
                log::error!("Failed to create settings webview: {e}");
                return;
            }
        };

        window_focus::present_and_activate(&settings_win);
        self.settings_focus_pending.set(true);
        *self.settings_window.borrow_mut() = Some(SettingsWindowState {
            window: settings_win,
            webview: settings_webview,
        });
    }

    fn close_settings_window(&self) {
        if let Some(settings) = self.settings_window.borrow_mut().take() {
            settings.window.set_visible(false);
            drop(settings);
        }
    }

    fn on_settings_focused(&self) {
        if !self.settings_focus_pending.take() {
            return;
        }
        if let Some(settings) = self.settings_window.borrow().as_ref() {
            let _ = settings.webview.focus_parent();
            let _ = settings.webview.focus();
            settings.window.set_focus();
        }
    }

    fn notify_frontend_config_updated(&self) {
        let _ = self.webview.evaluate_script(
            "window.__promptlyConfigUpdated && window.__promptlyConfigUpdated();",
        );
    }

    fn deliver_ipc_response(&self, source: IpcSource, response_json: &str) {
        let script = format!("window.__promptlyReceive({response_json});");
        match source {
            IpcSource::Main => {
                let _ = self.webview.evaluate_script(&script);
            }
            IpcSource::Settings => {
                if let Some(settings) = self.settings_window.borrow().as_ref() {
                    let _ = settings.webview.evaluate_script(&script);
                }
            }
        }
    }

    fn push_notifications(&self, notifications: &[AppNotification]) {
        if notifications.is_empty() {
            return;
        }
        let Ok(json) = serde_json::to_string(notifications) else {
            log::error!("Failed to serialize in-app notifications");
            return;
        };
        let script = format!(
            "window.__promptlyPushNotifications && window.__promptlyPushNotifications({json});"
        );
        if let Err(e) = self.webview.evaluate_script(&script) {
            log::error!("Failed to push in-app notifications to frontend: {e}");
        }
    }

    fn spawn_update_check(&self, user_initiated: bool) {
        let proxy = self.event_proxy.clone();
        let ephemeral_ms = self.ephemeral_notification_ms();
        std::thread::spawn(move || {
            let outcome = crate::update::check_for_updates().map_err(|e| e.to_string());
            if let Err(e) = proxy.send_event(AppEvent::UpdateCheckResult {
                outcome: outcome.clone(),
                user_initiated,
            }) {
                log::error!("Failed to post update check result to event loop: {e}");
                if user_initiated {
                    let collector = NotificationCollector::new(ephemeral_ms);
                    Self::deliver_update_check_notifications(
                        &collector,
                        outcome,
                        true,
                    );
                    let _ = proxy.send_event(AppEvent::PushNotifications(collector.take()));
                }
            }
        });
    }

    fn handle_update_check_result(
        &self,
        outcome: Result<UpdateCheckOutcome, String>,
        user_initiated: bool,
    ) {
        let collector = NotificationCollector::new(self.ephemeral_notification_ms());
        Self::deliver_update_check_notifications(&collector, outcome, user_initiated);
        self.push_notifications(&collector.take());
    }

    fn deliver_update_check_notifications(
        effects: &NotificationCollector,
        outcome: Result<UpdateCheckOutcome, String>,
        user_initiated: bool,
    ) {
        match outcome {
            Ok(UpdateCheckOutcome::UpdateAvailable(info)) => {
                effects.notify_update_available(&info);
            }
            Ok(UpdateCheckOutcome::UpToDate { latest, .. }) => {
                if user_initiated {
                    effects.notify_up_to_date(&latest);
                }
            }
            Err(e) => {
                log::warn!("Update check failed: {e}");
                if user_initiated {
                    effects.notify_update_check_failed(&e);
                }
            }
        }
    }

    fn handle_ipc(&self, raw: &str, source: IpcSource, target: &EventLoopWindowTarget<AppEvent>) {
        let effects = NotificationCollector::new(self.ephemeral_notification_ms());
        let handled = self.backend.handle(raw, &effects);
        let notifications = effects.take();

        self.deliver_ipc_response(source, &handled.response_json);

        if let Some(title) = handled.window_title {
            if source == IpcSource::Main {
                self.window.set_title(&title);
            }
        }

        if handled.open_settings_window {
            self.open_settings_window(target);
        }
        if handled.close_settings_window {
            self.close_settings_window();
        } else if handled.hide_window {
            match source {
                IpcSource::Main => self.hide_window(),
                IpcSource::Settings => self.close_settings_window(),
            }
        }
        if handled.config_changed {
            self.notify_frontend_config_updated();
        }
        if handled.run_update {
            std::thread::spawn(|| {
                if let Err(e) = crate::update::run_update() {
                    log::error!("Update failed: {e:#}");
                }
            });
        }
        if handled.quit_app {
            std::process::exit(0);
        }
        if source == IpcSource::Main {
            self.push_notifications(&notifications);
        }
    }
}
