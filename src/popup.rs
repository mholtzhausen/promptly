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
use crate::window_hints;

/// Callback when a prompt is selected.
pub type OnPromptSelect = Rc<dyn Fn(&Prompt)>;
/// Callback for prompt edit/delete actions.
pub type OnPromptAction = Rc<dyn Fn(Prompt)>;
/// Callback to open the "New Prompt" dialog.
pub type OnAddClick = Rc<dyn Fn()>;

/// The main popup window widget.
pub struct PopupWindow {
    pub window: ApplicationWindow,
    search_entry: SearchEntry,
    prompt_list: ListBox,
    status_label: Label,
    prompts: Rc<RefCell<Vec<Prompt>>>,
    on_prompt_select: OnPromptSelect,
    on_edit_click: OnPromptAction,
    on_delete_click: OnPromptAction,
}

impl PopupWindow {
    /// Create a new popup window.
    pub fn new(
        app: &Application,
        conn: Connection,
        on_prompt_select: OnPromptSelect,
        on_add_click: OnAddClick,
        on_edit_click: OnPromptAction,
        on_delete_click: OnPromptAction,
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

        // Search entry + add button
        let top_bar = GtkBox::new(Orientation::Horizontal, 6);
        top_bar.set_hexpand(true);
        top_bar.set_margin_top(6);
        top_bar.set_margin_end(6);
        top_bar.set_margin_start(6);
        top_bar.set_margin_bottom(4);

        let search_entry = SearchEntry::builder()
            .name("search-entry")
            .placeholder_text("Filter prompts...")
            .hexpand(true)
            .build();

        let add_btn = Button::builder().name("add-button").label("+").build();
        add_btn.set_tooltip_text(Some("Add prompt"));
        add_btn.connect_clicked(move |_| {
            on_add_click();
        });
        top_bar.append(&search_entry);
        top_bar.append(&add_btn);
        main_box.append(&top_bar);

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
            window_hints::apply_now(window, POPUP_DEFAULT_WIDTH, POPUP_DEFAULT_HEIGHT);
            gtk4::prelude::GtkWindowExt::set_focus(window, Some(&map_search_entry));
            map_search_entry.grab_focus();
        });

        // ── Selection callbacks (Enter + click) ──────────────────────────
        let prompts_rc = Rc::new(RefCell::new(Vec::<Prompt>::new()));
        let select_cb = Rc::clone(&on_prompt_select);
        let prompts_clone = Rc::clone(&prompts_rc);
        let prompt_list_clone = prompt_list.clone();

        // Enter key on the list activates selected row.
        let list_key_ctrl = EventControllerKey::new();
        list_key_ctrl.connect_key_pressed(move |_, key, _, _| {
            if key == gdk::Key::Return {
                if let Some(row) = prompt_list_clone.selected_row() {
                    let id = row.widget_name().parse::<i64>().ok();
                    let prompts = prompts_clone.borrow();
                    if let Some(id) = id {
                        if let Some(prompt) = prompts.iter().find(|prompt| prompt.id == id) {
                            select_cb(prompt);
                        }
                    }
                }
                glib::Propagation::Stop
            } else {
                glib::Propagation::Proceed
            }
        });
        prompt_list.add_controller(list_key_ctrl);

        let mut popup = Self {
            window,
            search_entry: search_entry.clone(),
            prompt_list: prompt_list.clone(),
            status_label: status_label.clone(),
            prompts: Rc::clone(&prompts_rc),
            on_prompt_select: Rc::clone(&on_prompt_select),
            on_edit_click: Rc::clone(&on_edit_click),
            on_delete_click: Rc::clone(&on_delete_click),
        };

        // Filter search entry changes

        let prompt_list_filter = prompt_list.clone();
        let search_entry_clone = search_entry.clone();
        let status_label_clone = status_label.clone();
        let prompts_filter_clone = Rc::clone(&prompts_rc);
        let select_filter = Rc::clone(&on_prompt_select);
        let edit_filter = Rc::clone(&on_edit_click);
        let delete_filter = Rc::clone(&on_delete_click);
        search_entry.connect_search_changed(move |_| {
            Self::rebuild_list(
                &prompt_list_filter,
                &search_entry_clone,
                &status_label_clone,
                &prompts_filter_clone,
                &select_filter,
                &edit_filter,
                &delete_filter,
            );
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
        Self::rebuild_list(
            &self.prompt_list,
            &self.search_entry,
            &self.status_label,
            &self.prompts,
            &self.on_prompt_select,
            &self.on_edit_click,
            &self.on_delete_click,
        );
    }

    fn rebuild_list(
        prompt_list: &ListBox,
        search_entry: &SearchEntry,
        status_label: &Label,
        prompts: &Rc<RefCell<Vec<Prompt>>>,
        on_select: &OnPromptSelect,
        on_edit: &OnPromptAction,
        on_delete: &OnPromptAction,
    ) {
        while let Some(child) = prompt_list.first_child() {
            prompt_list.remove(&child);
        }

        let query = search_entry.text().to_string();
        let prompts = prompts.borrow();

        let filtered: Vec<&Prompt> = if query.is_empty() {
            prompts.iter().collect()
        } else {
            prompts
                .iter()
                .filter(|prompt| {
                    fuzzy_match(&prompt.name, &query)
                        || fuzzy_match(&prompt.description, &query)
                        || fuzzy_match(&prompt.content, &query)
                })
                .collect()
        };

        for prompt in &filtered {
            let row = Self::create_prompt_row(prompt, on_select, on_edit, on_delete);
            prompt_list.append(&row);
        }

        let count = filtered.len();
        if prompts.is_empty() {
            status_label.set_text("No prompts yet. Click + to add one.");
        } else if query.is_empty() || !filtered.is_empty() {
            status_label.set_text(&format!(
                "{count} prompt{plural} available",
                plural = if count != 1 { "s" } else { "" }
            ));
        } else {
            status_label.set_text(&format!("No matches for \"{query}\""));
        }
    }

    /// Create a ListBoxRow for a prompt.
    fn create_prompt_row(
        prompt: &Prompt,
        on_select: &OnPromptSelect,
        on_edit: &OnPromptAction,
        on_delete: &OnPromptAction,
    ) -> ListBoxRow {
        let row = ListBoxRow::new();
        row.set_widget_name(&prompt.id.to_string());

        let row_box = GtkBox::new(Orientation::Horizontal, 8);
        row_box.add_css_class("prompt-row");

        let text_box = GtkBox::new(Orientation::Horizontal, 8);
        text_box.set_hexpand(true);
        text_box.set_valign(Align::Center);

        let name_label = Label::builder()
            .label(&prompt.name)
            .halign(Align::Start)
            .ellipsize(pango::EllipsizeMode::End)
            .build();
        name_label.add_css_class("prompt-title");

        let description_label = Label::builder()
            .label(&prompt.description)
            .halign(Align::Start)
            .hexpand(true)
            .ellipsize(pango::EllipsizeMode::End)
            .build();
        description_label.add_css_class("prompt-description");

        text_box.append(&name_label);
        text_box.append(&description_label);

        let select_prompt = prompt.clone();
        let select_cb = Rc::clone(on_select);
        let select_gesture = gtk4::GestureClick::new();
        select_gesture.connect_released(move |_, n_press, _, _| {
            if n_press == 1 {
                select_cb(&select_prompt);
            }
        });
        text_box.add_controller(select_gesture);

        let action_box = GtkBox::new(Orientation::Horizontal, 2);
        action_box.add_css_class("prompt-actions");

        let edit_btn = Button::builder()
            .name("prompt-edit-button")
            .icon_name("document-edit-symbolic")
            .has_frame(false)
            .build();
        edit_btn.set_tooltip_text(Some("Edit prompt"));
        let edit_prompt = prompt.clone();
        let edit_cb = Rc::clone(on_edit);
        edit_btn.connect_clicked(move |_| {
            edit_cb(edit_prompt.clone());
        });

        let delete_btn = Button::builder()
            .name("prompt-delete-button")
            .icon_name("edit-delete-symbolic")
            .has_frame(false)
            .build();
        delete_btn.set_tooltip_text(Some("Delete prompt"));
        let delete_prompt = prompt.clone();
        let delete_cb = Rc::clone(on_delete);
        delete_btn.connect_clicked(move |_| {
            delete_cb(delete_prompt.clone());
        });

        action_box.append(&edit_btn);
        action_box.append(&delete_btn);
        row_box.append(&text_box);
        row_box.append(&action_box);
        row.set_child(Some(&row_box));

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
        window_hints::present_centered_always_on_top(
            &self.window,
            POPUP_DEFAULT_WIDTH,
            POPUP_DEFAULT_HEIGHT,
            Some(&self.search_entry),
        );
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
