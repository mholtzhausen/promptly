//! Prompt Tray Manager — system-tray app for managing prompt templates.
//!
//! Global shortcut: Ctrl+Alt+Space toggles the popup window.
//! Variables syntax: {{name|type|default|description}}

mod config;
mod db;
mod popup;
mod prompt_parser;
mod tray;
mod variable_dialog;
mod window_hints;

use anyhow::Result;
use gtk4::prelude::*;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

fn main() -> Result<()> {
    env_logger::init();

    // ── Ensure config directory exists ───────────────────────────────
    config::ensure_config_dir()?;

    // ── GTK Application (hidden window for tray + popup) ─────────────
    let app = gtk4::Application::builder()
        .application_id("com.prompt_tray.app")
        .build();
    // Keep the background app alive even when no GTK window is visible.
    let _hold_guard = app.hold();

    // ── Popup state ──────────────────────────────────────────────────
    let popup_state: Arc<Mutex<Option<popup::PopupWindow>>> = Arc::new(Mutex::new(None));

    // Channel for hotkey triggers from rdev thread to main thread
    let (tx, rx) = std::sync::mpsc::channel::<()>();
    let rx_holder = Arc::new(Mutex::new(Some(rx)));

    let popup_state_clone = Arc::clone(&popup_state);
    let rx_holder_clone = Arc::clone(&rx_holder);

    let tx_clone = tx.clone();
    app.connect_activate(move |app| {
        // Initialize tray icon inside connect_activate on the main thread
        let tray_state = match tray::TrayState::build(tx_clone.clone()) {
            Ok(t) => t,
            Err(e) => {
                log::error!("Failed to build tray icon: {}", e);
                std::process::exit(1);
            }
        };

        let ps = Arc::clone(&popup_state_clone);
        let app_clone = app.clone();

        // Keep the tray icon alive by moving it into the timeout closure
        let _tray = tray_state;

        let rx = rx_holder_clone
            .lock()
            .unwrap()
            .take()
            .expect("connect_activate called twice");
        let ps_timer = Arc::clone(&ps);
        let app_timer = app_clone.clone();

        gtk4::glib::timeout_add_local(std::time::Duration::from_millis(50), move || {
            let _keep_alive = &_tray;
            // Process hotkey triggers and tray menu show triggers
            while let Ok(_) = rx.try_recv() {
                handle_hotkey(&ps_timer, &app_timer);
            }
            gtk4::glib::ControlFlow::Continue
        });
    });

    // ── Global hotkey registration via rdev (X11 + Wayland) ──────────
    register_global_hotkey(tx);

    // ── Run GTK main loop ────────────────────────────────────────────
    app.run();

    Ok(())
}

/// Register the global Ctrl+Alt+Space hotkey using rdev.
fn register_global_hotkey(tx: std::sync::mpsc::Sender<()>) {
    use rdev::{EventType, Key};

    // Track modifier state for Ctrl+Alt detection
    let ctrl_pressed = Arc::new(Mutex::new(false));
    let alt_pressed = Arc::new(Mutex::new(false));
    let cp = Arc::clone(&ctrl_pressed);
    let ap = Arc::clone(&alt_pressed);

    log::info!("Registering global hotkey Ctrl+Alt+Space...");

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
                "rdev listener error: {:?}. App will still work via tray menu.",
                e
            );
        }
    });
}

/// Handle the popup toggle when triggered.
fn handle_hotkey(popup_state: &Arc<Mutex<Option<popup::PopupWindow>>>, app: &gtk4::Application) {
    let should_show = {
        let mut guard = popup_state.lock().unwrap();
        match guard.as_mut() {
            Some(popup) => {
                if popup.is_visible() {
                    popup.hide();
                    false
                } else {
                    true
                }
            }
            None => true,
        }
    };

    if should_show {
        // Create new popup window
        let conn = db::init_db(&config::db_path()).unwrap_or_else(|e| {
            log::error!("Failed to open database: {}", e);
            std::process::exit(1);
        });

        let ps = Arc::clone(popup_state);
        let app_ref = app.clone();
        let ps_dialog = Arc::clone(popup_state);
        let ps_edit = Arc::clone(popup_state);
        let ps_delete = Arc::clone(popup_state);

        let select_cb: Rc<dyn Fn(&db::Prompt)> = Rc::new(move |prompt: &db::Prompt| {
            handle_prompt_select(prompt, &ps, &app_ref);
        });
        let add_cb: Rc<dyn Fn()> = Rc::new(move || {
            show_prompt_dialog(&ps_dialog, None);
        });
        let edit_cb: Rc<dyn Fn(db::Prompt)> = Rc::new(move |prompt: db::Prompt| {
            show_prompt_dialog(&ps_edit, Some(prompt));
        });
        let delete_cb: Rc<dyn Fn(db::Prompt)> = Rc::new(move |prompt: db::Prompt| {
            show_delete_confirmation(&ps_delete, prompt);
        });

        let popup_window =
            popup::PopupWindow::new(app, conn, select_cb, add_cb, edit_cb, delete_cb);

        popup_window.show();

        let mut guard = popup_state.lock().unwrap();
        *guard = Some(popup_window);
    }
}

/// Handle prompt selection: parse variables and show input dialog.
fn handle_prompt_select(
    prompt: &db::Prompt,
    popup_state: &Arc<Mutex<Option<popup::PopupWindow>>>,
    app: &gtk4::Application,
) {
    let variables = prompt_parser::parse_variables(&prompt.content);

    // Hide the main popup
    if let Some(popup) = popup_state.lock().unwrap().as_mut() {
        popup.hide();
    }

    if variables.is_empty() {
        // No variables — copy immediately and show notification
        let result = prompt.content.clone();
        match arboard::Clipboard::new() {
            Ok(mut clipboard) => {
                if let Err(e) = clipboard.set_text(&result) {
                    log::error!("Failed to copy: {}", e);
                }
            }
            Err(e) => {
                log::error!("Failed to create clipboard: {}", e);
            }
        }
        show_notification(
            "Prompt copied",
            &format!("'{}' copied to clipboard", prompt.name),
        );
    } else {
        // Show variable input dialog
        let ps = Arc::clone(popup_state);

        // Create a temporary window for the dialog parent
        let temp_window = gtk4::Window::builder()
            .application(app)
            .visible(false)
            .build();

        let name_string = prompt.name.clone();
        variable_dialog::show_variable_dialog(
            &temp_window,
            &prompt.name,
            &prompt.content,
            &variables,
            move |_result: &str| {
                show_notification(
                    "Prompt copied",
                    &format!("'{}' copied to clipboard!", name_string),
                );
                // Restore popup visibility
                let ps = Arc::clone(&ps);
                gtk4::glib::idle_add_local(move || {
                    if let Some(popup) = ps.lock().unwrap().as_mut() {
                        popup.show();
                    }
                    gtk4::glib::ControlFlow::Break
                });
            },
        );
    }
}

/// Show the prompt create/edit dialog.
fn show_prompt_dialog(
    popup_state: &Arc<Mutex<Option<popup::PopupWindow>>>,
    prompt: Option<db::Prompt>,
) {
    let parent = {
        let guard = popup_state.lock().unwrap();
        guard.as_ref().map(|p| p.window.clone())
    };
    let parent = match parent {
        Some(w) => w,
        None => return,
    };

    let editing_id = prompt.as_ref().map(|prompt| prompt.id);
    let dialog = gtk4::Dialog::builder()
        .application(&parent.application().unwrap())
        .title(if editing_id.is_some() {
            "Edit Prompt Template"
        } else {
            "New Prompt Template"
        })
        .transient_for(&parent)
        .modal(true)
        .build();

    let content_area = dialog.content_area();
    let main_box = gtk4::Box::new(gtk4::Orientation::Vertical, 10);
    main_box.set_margin_start(14);
    main_box.set_margin_end(14);
    main_box.set_margin_top(14);
    main_box.set_margin_bottom(14);

    // CSS styling
    let provider = gtk4::CssProvider::new();
    provider.load_from_data(config::CSS);
    gtk4::style_context_add_provider_for_display(
        &gtk4::prelude::WidgetExt::display(&dialog),
        &provider,
        gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );

    let name_label = gtk4::Label::builder()
        .label("Prompt Name")
        .halign(gtk4::Align::Start)
        .build();
    let name_entry = gtk4::Entry::builder()
        .placeholder_text("e.g. git-commit")
        .name("variable-entry")
        .build();

    let description_label = gtk4::Label::builder()
        .label("Description")
        .halign(gtk4::Align::Start)
        .build();
    let description_entry = gtk4::Entry::builder()
        .placeholder_text("Short summary shown next to the title")
        .name("variable-entry")
        .build();

    let template_label = gtk4::Label::builder()
        .label("Template Content")
        .halign(gtk4::Align::Start)
        .build();
    let template_buffer = gtk4::TextBuffer::new(None);
    let template_view = gtk4::TextView::builder()
        .buffer(&template_buffer)
        .name("template-textview")
        .wrap_mode(gtk4::WrapMode::WordChar)
        .hexpand(true)
        .vexpand(true)
        .build();
    let scrolled = gtk4::ScrolledWindow::builder()
        .child(&template_view)
        .min_content_height(150)
        .build();

    if let Some(prompt) = prompt.as_ref() {
        name_entry.set_text(&prompt.name);
        description_entry.set_text(&prompt.description);
        template_buffer.set_text(&prompt.content);
    }

    let help_label = gtk4::Label::builder()
        .label(
            "Use {{name|type|default|desc}} placeholders. Types: text, number, option, multiline.",
        )
        .name("help-label")
        .halign(gtk4::Align::Start)
        .build();

    main_box.append(&name_label);
    main_box.append(&name_entry);
    main_box.append(&description_label);
    main_box.append(&description_entry);
    main_box.append(&template_label);
    main_box.append(&scrolled);
    main_box.append(&help_label);

    let button_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
    button_box.set_halign(gtk4::Align::End);
    button_box.set_margin_top(14);

    let cancel_btn = gtk4::Button::builder()
        .name("cancel-button")
        .label("Cancel")
        .build();
    let dialog_cancel = dialog.clone();
    cancel_btn.connect_clicked(move |_| {
        dialog_cancel.close();
    });

    let save_btn = gtk4::Button::builder()
        .name("copy-button")
        .label(if editing_id.is_some() {
            "Update"
        } else {
            "Save"
        })
        .build();

    let dialog_save = dialog.clone();
    let name_entry_clone = name_entry.clone();
    let description_entry_clone = description_entry.clone();
    let popup_state_clone = Arc::clone(popup_state);
    save_btn.connect_clicked(move |_| {
        let name = name_entry_clone.text().trim().to_string();
        let description = description_entry_clone.text().trim().to_string();
        let start = template_buffer.start_iter();
        let end = template_buffer.end_iter();
        let content = template_buffer.text(&start, &end, false).to_string();

        if name.is_empty() || description.is_empty() || content.trim().is_empty() {
            return;
        }

        let conn = match db::init_db(&config::db_path()) {
            Ok(conn) => conn,
            Err(e) => {
                log::error!("Failed to open database: {}", e);
                return;
            }
        };

        let result = if let Some(id) = editing_id {
            db::update_prompt(&conn, id, &name, &description, &content).map(|_| id)
        } else {
            db::upsert_prompt(&conn, &name, &description, &content)
        };

        if let Err(e) = result {
            log::error!("Failed to save prompt: {}", e);
            show_notification("Prompt not saved", "Could not save the prompt template.");
            return;
        }

        show_notification("Prompt Saved", &format!("Saved template '{}'", name));
        refresh_popup_list(&popup_state_clone, &conn);
        dialog_save.close();
    });

    button_box.append(&cancel_btn);
    button_box.append(&save_btn);
    main_box.append(&button_box);

    content_area.append(&main_box);
    dialog.show();
}

fn show_delete_confirmation(
    popup_state: &Arc<Mutex<Option<popup::PopupWindow>>>,
    prompt: db::Prompt,
) {
    let parent = {
        let guard = popup_state.lock().unwrap();
        guard.as_ref().map(|p| p.window.clone())
    };
    let parent = match parent {
        Some(w) => w,
        None => return,
    };

    let dialog = gtk4::Dialog::builder()
        .application(&parent.application().unwrap())
        .title("Delete Prompt Template")
        .transient_for(&parent)
        .modal(true)
        .build();
    let content_area = dialog.content_area();
    let main_box = gtk4::Box::new(gtk4::Orientation::Vertical, 12);
    main_box.set_margin_start(14);
    main_box.set_margin_end(14);
    main_box.set_margin_top(14);
    main_box.set_margin_bottom(14);

    let message = gtk4::Label::builder()
        .label(&format!("Delete '{}'? This cannot be undone.", prompt.name))
        .wrap(true)
        .halign(gtk4::Align::Start)
        .build();
    main_box.append(&message);

    let button_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
    button_box.set_halign(gtk4::Align::End);

    let cancel_btn = gtk4::Button::builder()
        .name("cancel-button")
        .label("Cancel")
        .build();
    let dialog_cancel = dialog.clone();
    cancel_btn.connect_clicked(move |_| {
        dialog_cancel.close();
    });

    let delete_btn = gtk4::Button::builder()
        .name("prompt-delete-confirm-button")
        .label("Delete")
        .build();
    let dialog_delete = dialog.clone();
    let popup_state_delete = Arc::clone(popup_state);
    delete_btn.connect_clicked(move |_| {
        let conn = match db::init_db(&config::db_path()) {
            Ok(conn) => conn,
            Err(e) => {
                log::error!("Failed to open database: {}", e);
                return;
            }
        };

        if let Err(e) = db::delete_prompt(&conn, prompt.id) {
            log::error!("Failed to delete prompt: {}", e);
            show_notification(
                "Prompt not deleted",
                "Could not delete the prompt template.",
            );
            return;
        }

        show_notification(
            "Prompt deleted",
            &format!("Deleted template '{}'", prompt.name),
        );
        refresh_popup_list(&popup_state_delete, &conn);
        dialog_delete.close();
    });

    button_box.append(&cancel_btn);
    button_box.append(&delete_btn);
    main_box.append(&button_box);
    content_area.append(&main_box);
    dialog.show();
}

fn refresh_popup_list(popup_state: &Arc<Mutex<Option<popup::PopupWindow>>>, conn: &db::Connection) {
    let mut guard = popup_state.lock().unwrap();
    if let Some(popup) = guard.as_mut() {
        popup.refresh_prompts(conn);
    }
}

/// Show a desktop notification.
fn show_notification(summary: &str, body: &str) {
    use notify_rust::Notification;
    if let Err(e) = Notification::new().summary(summary).body(body).show() {
        log::error!("Failed to show notification: {}", e);
    }
}
