//! Popup window with fuzzy-searchable prompt list.

use gtk4::prelude::*;
use gtk4::{
    gdk, pango, Align, Application, ApplicationWindow, Box as GtkBox, Button, EventControllerKey,
    Label, ListBox, ListBoxRow, Orientation, ScrolledWindow, SearchEntry,
};

use std::cell::RefCell;
use std::rc::Rc;

const POPUP_DEFAULT_WIDTH: i32 = 500;
const POPUP_DEFAULT_HEIGHT: i32 = 400;

use crate::config::CSS;
use crate::db::{self, Connection, Prompt};

/// Callback when a prompt is selected (name + content).
pub type OnPromptSelect = Rc<dyn Fn(&str, &str)>;
/// Callback to open the "New Prompt" dialog.
pub type OnAddClick = Rc<dyn Fn()>;

/// The main popup window widget.
pub struct PopupWindow {
    pub window: ApplicationWindow,
    search_entry: SearchEntry,
    prompt_list: ListBox,
    status_label: Label,
    prompts: Rc<RefCell<Vec<Prompt>>>,
}

impl PopupWindow {
    /// Create a new popup window.
    pub fn new(
        app: &Application,
        conn: Connection,
        on_prompt_select: OnPromptSelect,
        on_add_click: OnAddClick,
    ) -> Self {
        let window = ApplicationWindow::builder()
            .application(app)
            .title("Prompt Manager")
            .name("popup-window")
            .default_width(POPUP_DEFAULT_WIDTH)
            .default_height(POPUP_DEFAULT_HEIGHT)
            .decorated(true)
            .hide_on_close(true)
            .modal(true)
            .build();

        // ── CSS styling ──────────────────────────────────────────────────
        let provider = gtk4::CssProvider::new();
        provider.load_from_data(CSS);
        gtk4::style_context_add_provider_for_display(
            &gtk4::prelude::WidgetExt::display(&window),
            &provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );

        // ── Layout ───────────────────────────────────────────────────────
        let main_box = GtkBox::new(Orientation::Vertical, 0);
        window.set_child(Some(&main_box));

        // Top bar with + button
        let top_bar = GtkBox::new(Orientation::Horizontal, 0);
        top_bar.set_hexpand(true);
        top_bar.set_margin_top(8);
        top_bar.set_margin_end(8);
        top_bar.set_margin_start(8);

        let add_btn = Button::builder().name("add-button").label("+").build();
        add_btn.connect_clicked(move |_| {
            on_add_click();
        });
        top_bar.append(&add_btn);
        main_box.append(&top_bar);

        // Search entry
        let search_entry = SearchEntry::builder()
            .name("search-entry")
            .placeholder_text("Type to filter prompts...")
            .build();

        // Prompt list in a scrolled window
        let prompt_list = ListBox::builder()
            .name("prompt-list")
            .selection_mode(gtk4::SelectionMode::Single)
            .build();

        let scrolled = ScrolledWindow::builder()
            .child(&prompt_list)
            .vexpand(true)
            .build();

        // Status label
        let status_label = Label::builder()
            .name("status-label")
            .label("0 prompts available")
            .halign(Align::Start)
            .build();

        main_box.append(&search_entry);
        main_box.append(&scrolled);
        main_box.append(&status_label);

        // ── Escape/window close hides back to the tray ────────────────────
        let escape_controller = EventControllerKey::new();
        let window_clone = window.clone();
        escape_controller.connect_key_pressed(move |_, key, _, _| {
            if key == gdk::Key::Escape {
                window_clone.set_visible(false);
                glib::Propagation::Stop
            } else {
                glib::Propagation::Proceed
            }
        });
        window.add_controller(escape_controller);

        window.connect_close_request(|window| {
            window.set_visible(false);
            glib::Propagation::Stop
        });

        let map_search_entry = search_entry.clone();
        window.connect_map(move |window| {
            center_window_on_screen(window);
            keep_window_above(window);
            gtk4::prelude::GtkWindowExt::set_focus(window, Some(&map_search_entry));
            map_search_entry.grab_focus();
        });

        // ── Selection callbacks (Enter + click) ──────────────────────────
        let prompts_rc = Rc::new(RefCell::new(Vec::<Prompt>::new()));
        let select_cb = Rc::clone(&on_prompt_select);
        let prompts_clone = Rc::clone(&prompts_rc);
        let prompt_list_clone = prompt_list.clone();

        // Enter key on the list activates selected row
        let list_key_ctrl = EventControllerKey::new();
        list_key_ctrl.connect_key_pressed(move |_, key, _, _| {
            if key == gdk::Key::Return {
                if let Some(row) = prompt_list_clone.selected_row() {
                    let name = row.widget_name().to_string();
                    let prompts = prompts_clone.borrow();
                    if let Some(prompt) = prompts.iter().find(|p| p.name == name) {
                        select_cb(&prompt.name, &prompt.content);
                    }
                }
                glib::Propagation::Stop
            } else {
                glib::Propagation::Proceed
            }
        });
        prompt_list.add_controller(list_key_ctrl);

        // Single-click activates selected row
        let select_cb2 = Rc::clone(&on_prompt_select);
        let prompts_clone2 = Rc::clone(&prompts_rc);
        let prompt_list_clone2 = prompt_list.clone();
        let click_gesture = gtk4::GestureClick::new();
        click_gesture.connect_released(move |_, n_press, _, _| {
            if n_press == 1 {
                if let Some(row) = prompt_list_clone2.selected_row() {
                    let name = row.widget_name().to_string();
                    let prompts = prompts_clone2.borrow();
                    if let Some(prompt) = prompts.iter().find(|p| p.name == name) {
                        select_cb2(&prompt.name, &prompt.content);
                    }
                }
            }
        });
        prompt_list.add_controller(click_gesture);

        // ── State & construction ─────────────────────────────────────────
        let mut popup = Self {
            window,
            search_entry: search_entry.clone(),
            prompt_list: prompt_list.clone(),
            status_label: status_label.clone(),
            prompts: Rc::clone(&prompts_rc),
        };

        // Filter search entry changes

        let prompt_list_filter = prompt_list.clone();
        let search_entry_clone = search_entry.clone();
        let status_label_clone = status_label.clone();
        let prompts_filter_clone = Rc::clone(&prompts_rc);
        search_entry.connect_search_changed(move |_| {
            // Re-run the update list logic manually in closure
            while let Some(child) = prompt_list_filter.first_child() {
                prompt_list_filter.remove(&child);
            }
            let query = search_entry_clone.text().to_string();
            let prompts = prompts_filter_clone.borrow();
            let filtered: Vec<&Prompt> = if query.is_empty() {
                prompts.iter().collect()
            } else {
                prompts
                    .iter()
                    .filter(|p| fuzzy_match(&p.name, &query) || fuzzy_match(&p.content, &query))
                    .collect()
            };
            for prompt in &filtered {
                let row = Self::create_prompt_row(prompt);
                prompt_list_filter.append(&row);
            }
            let count = filtered.len();
            if prompts.is_empty() {
                status_label_clone.set_text("No prompts yet. Click + to add one.");
            } else if query.is_empty() || !filtered.is_empty() {
                status_label_clone.set_text(&format!(
                    "{count} prompt{plural} available",
                    plural = if count != 1 { "s" } else { "" }
                ));
            } else {
                status_label_clone.set_text(&format!("No matches for \"{query}\""));
            }
        });

        // Load initial prompts from DB
        popup.refresh_prompts(&conn);

        popup
    }

    /// Refresh the prompt list from the database.
    pub fn refresh_prompts(&mut self, conn: &Connection) {
        *self.prompts.borrow_mut() = db::load_prompts(conn).unwrap_or_default();
        self.update_list();
    }

    /// Update the visible list based on current search text.
    pub fn update_list(&self) {
        while let Some(child) = self.prompt_list.first_child() {
            self.prompt_list.remove(&child);
        }

        let query = self.search_entry.text().to_string();
        let prompts = self.prompts.borrow();

        let filtered: Vec<&Prompt> = if query.is_empty() {
            prompts.iter().collect()
        } else {
            prompts
                .iter()
                .filter(|p| fuzzy_match(&p.name, &query) || fuzzy_match(&p.content, &query))
                .collect()
        };

        for prompt in &filtered {
            let row = Self::create_prompt_row(prompt);
            self.prompt_list.append(&row);
        }

        let count = filtered.len();
        if prompts.is_empty() {
            self.status_label
                .set_text("No prompts yet. Click + to add one.");
        } else if query.is_empty() || !filtered.is_empty() {
            self.status_label.set_text(&format!(
                "{count} prompt{plural} available",
                plural = if count != 1 { "s" } else { "" }
            ));
        } else {
            self.status_label
                .set_text(&format!("No matches for \"{query}\""));
        }
    }

    /// Create a ListBoxRow for a prompt.
    fn create_prompt_row(prompt: &Prompt) -> ListBoxRow {
        let row = ListBoxRow::new();
        let box_ = GtkBox::new(Orientation::Horizontal, 8);
        box_.set_margin_start(12);
        box_.set_margin_end(12);
        box_.set_margin_top(6);
        box_.set_margin_bottom(6);

        // Name label (bold)
        let name_label = Label::builder()
            .label(&prompt.name)
            .halign(Align::Start)
            .hexpand(true)
            .ellipsize(pango::EllipsizeMode::End)
            .build();

        // Preview (first 60 chars of content)
        let preview = if prompt.content.len() > 60 {
            &prompt.content[..60]
        } else {
            &prompt.content
        };
        let preview_label = Label::builder()
            .label(preview)
            .halign(Align::Start)
            .ellipsize(pango::EllipsizeMode::End)
            .build();

        box_.append(&name_label);
        box_.append(&preview_label);
        row.set_child(Some(&box_));

        // Store prompt name on the row widget for selection callbacks
        row.set_widget_name(&prompt.name);

        row
    }

    /// Show the popup window and focus the search entry.
    pub fn show(&self) {
        self.update_list();
        self.window.present();
        gtk4::prelude::GtkWindowExt::set_focus(&self.window, Some(&self.search_entry));
        self.search_entry.grab_focus();
        self.apply_popup_window_hints();
    }

    fn apply_popup_window_hints(&self) {
        let window = self.window.clone();
        let search_entry = self.search_entry.clone();

        glib::idle_add_local_once(move || {
            center_window_on_screen(&window);
            keep_window_above(&window);
            window.present();
            gtk4::prelude::GtkWindowExt::set_focus(&window, Some(&search_entry));
            search_entry.grab_focus();
        });
    }

    /// Hide the popup window.
    pub fn hide(&self) {
        self.window.set_visible(false);
    }

    /// Check if the popup is currently visible.
    pub fn is_visible(&self) -> bool {
        self.window.is_visible()
    }
}

#[cfg(target_os = "linux")]
extern "C" {
    fn gdk_x11_surface_get_xid(surface: *mut gdk::ffi::GdkSurface) -> std::os::raw::c_ulong;
}

#[cfg(target_os = "linux")]
fn x11_window_id(window: &ApplicationWindow) -> Option<std::os::raw::c_ulong> {
    use gtk4::glib::translate::ToGlibPtr;

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
fn center_window_on_screen(window: &ApplicationWindow) {
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
    let width = window.width().max(POPUP_DEFAULT_WIDTH);
    let height = window.height().max(POPUP_DEFAULT_HEIGHT);
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
fn center_window_on_screen(_: &ApplicationWindow) {}

#[cfg(target_os = "linux")]
fn keep_window_above(window: &ApplicationWindow) {
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
fn keep_window_above(_: &ApplicationWindow) {}

/// Simple fuzzy matching: checks if characters of `pattern` appear in order in `text`.
fn fuzzy_match(text: &str, pattern: &str) -> bool {
    let text_lower = text.to_lowercase();
    let pattern_lower = pattern.to_lowercase();
    let mut chars = text_lower.chars();
    for pat_char in pattern_lower.chars() {
        match chars.find(|c| c.to_ascii_lowercase() == pat_char) {
            Some(_) => continue,
            None => return false,
        }
    }
    true
}
