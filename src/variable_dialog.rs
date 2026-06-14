//! Variable input dialog with type-aware widgets and clipboard copy.

use gtk4::prelude::*;
use gtk4::{
    gdk, glib, Align, Box as GtkBox, Button, ComboBoxText, Dialog, Entry, EventControllerKey, Label,
    Orientation, ScrolledWindow, SpinButton, TextBuffer, TextView, Window,
};
use std::rc::Rc;

use crate::config::CSS;
use crate::prompt_parser::{interpolate, VarType, Variable};
use crate::window_hints;

const VARIABLE_DIALOG_WIDTH: i32 = 460;
const VARIABLE_DIALOG_HEIGHT: i32 = 560;
const PROMPT_EDITOR_HEIGHT: i32 = 150;

const VARIABLE_MULTILINE_HEIGHT: i32 = 100;
/// Show a variable input dialog for the given template and variables.
/// On submit, calls `on_copy` with the interpolated result.
pub fn show_variable_dialog(
    parent: &Window,
    prompt_name: &str,
    template: &str,
    variables: &[Variable],
    on_copy: impl Fn(&str) + 'static,
) {
    let dialog = Dialog::builder()
        .application(&parent.application().unwrap())
        .title(format!("Fill in variables for '{}'", prompt_name))
        .modal(true)
        .default_width(VARIABLE_DIALOG_WIDTH)
        .build();
    // ── Escape key closes dialog ──────────────────────────────────────
    let escape_controller = EventControllerKey::new();
    let dialog_esc = dialog.clone();
    escape_controller.connect_key_pressed(move |_, key, _, _| {
        if key == gdk::Key::Escape {
            dialog_esc.close();
            glib::Propagation::Stop
        } else {
            glib::Propagation::Proceed
        }
    });
    dialog.add_controller(escape_controller);
    // Cap dialog height to 85% of the parent monitor's work area
    let default_height = parent
        .surface()
        .and_then(|sfc| {
            let display = gtk4::prelude::WidgetExt::display(parent);
            let monitor = display.monitor_at_surface(&sfc)?;
            let geo = monitor.geometry();
            Some(VARIABLE_DIALOG_HEIGHT.min((geo.height() as f64 * 0.85) as i32))
        })
        .unwrap_or(VARIABLE_DIALOG_HEIGHT);
    dialog.set_default_size(VARIABLE_DIALOG_WIDTH, default_height);

    // ── CSS styling ──────────────────────────────────────────────────
    let provider = gtk4::CssProvider::new();
    provider.load_from_data(CSS);
    gtk4::style_context_add_provider_for_display(
        &gtk4::prelude::WidgetExt::display(&dialog),
        &provider,
        gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );

    // ── Layout ───────────────────────────────────────────────────────
    let content_area = dialog.content_area();
    let main_box = GtkBox::new(Orientation::Vertical, 10);
    main_box.set_margin_start(14);
    main_box.set_margin_end(14);
    main_box.set_margin_top(14);
    main_box.set_margin_bottom(14);

    // Variable input widgets — wrapped in a ScrolledWindow to scroll when too many fields
    let mut inputs = Vec::new();
    let input_box = GtkBox::new(Orientation::Vertical, 0);
    for var in variables {
        let var_box = GtkBox::new(Orientation::Vertical, 4);
        var_box.set_margin_start(2);

        let var_label = Label::builder()
            .label(&var.name)
            .halign(Align::Start)
            .build();
        var_label.add_css_class("variable-label");

        if !var.description.is_empty() {
            let description_label = Label::builder()
                .label(&var.description)
                .halign(Align::Start)
                .build();
            description_label.add_css_class("variable-description");
            var_box.append(&var_label);
            var_box.append(&description_label);
        } else {
            var_box.append(&var_label);
        }

        let input_widget: gtk4::Widget = match &var.var_type {
            VarType::Text => {
                let entry = Entry::builder()
                    .name("variable-entry")
                    .text(&var.default_value)
                    .hexpand(true)
                    .build();
                entry.into()
            }
            VarType::Number => {
                let adj = gtk4::Adjustment::builder()
                    .value(var.default_value.parse::<f64>().unwrap_or(0.0))
                    .lower(f64::MIN)
                    .upper(f64::MAX)
                    .step_increment(1.0)
                    .build();
                let spin = SpinButton::builder()
                    .adjustment(&adj)
                    .name("variable-entry")
                    .hexpand(true)
                    .build();
                spin.into()
            }
            VarType::Option(options) => {
                let combo = ComboBoxText::builder()
                    .name("variable-entry")
                    .hexpand(true)
                    .build();
                for opt in options {
                    combo.append_text(opt);
                }
                if !var.default_value.is_empty() && options.contains(&var.default_value) {
                    combo.set_active_id(Some(&var.default_value));
                } else if !options.is_empty() {
                    ComboBoxExtManual::set_active(&combo, Some(0));
                }
                combo.into()
            }
            VarType::Multiline => {
                let buffer = TextBuffer::builder().text(&var.default_value).build();
                let text_view = TextView::builder()
                    .buffer(&buffer)
                    .name("variable-textview")
                    .wrap_mode(gtk4::WrapMode::WordChar)
                    .accepts_tab(false)
                    .hexpand(true)
                    .vexpand(true)
                    .build();
                let text_view_clone = text_view.clone();
                let scrolled = ScrolledWindow::builder()
                    .child(&text_view)
                    .min_content_height(VARIABLE_MULTILINE_HEIGHT)
                    .propagate_natural_width(true)
                    .build();
                // Keep the TextView reference for input tracking (downcast target)
                // but append the ScrolledWindow to the layout.
                var_box.append(&scrolled);
                input_box.append(&var_box);
                inputs.push((var.name.clone(), var.var_type.clone(), text_view_clone.upcast::<gtk4::Widget>()));
                continue;
            }
        };

        var_box.append(&input_widget);
        input_box.append(&var_box);
        inputs.push((var.name.clone(), var.var_type.clone(), input_widget));
    }

    let first_input = inputs.first().map(|(_, _, widget)| widget.clone());
    let inputs = Rc::new(inputs);
    let input_scrolled = ScrolledWindow::builder()
        .child(&input_box)
        .vexpand(true)
        .propagate_natural_width(true)
        .build();
    main_box.append(&input_scrolled);

    let prompt_label = Label::builder()
        .label("Prompt to copy")
        .halign(Align::Start)
        .build();
    prompt_label.add_css_class("variable-label");

    let prompt_buffer = TextBuffer::new(None);
    set_prompt_editor_text(template, &inputs, &prompt_buffer);
    let prompt_view = TextView::builder()
        .buffer(&prompt_buffer)
        .name("prompt-preview-textview")
        .wrap_mode(gtk4::WrapMode::WordChar)
        .accepts_tab(false)
        .hexpand(true)
        .vexpand(false)
        .build();
    let prompt_scrolled = ScrolledWindow::builder()
        .child(&prompt_view)
        .min_content_height(PROMPT_EDITOR_HEIGHT)
        .max_content_height(PROMPT_EDITOR_HEIGHT)
        .vexpand(false)
        .build();

    attach_prompt_update_handlers(template.to_string(), Rc::clone(&inputs), &prompt_buffer);

    main_box.append(&prompt_label);
    main_box.append(&prompt_scrolled);

    // Buttons row
    let button_box = GtkBox::new(Orientation::Horizontal, 8);
    button_box.set_halign(Align::End);
    button_box.set_margin_top(8);

    let cancel_btn = Button::builder()
        .name("cancel-button")
        .label("Cancel")
        .build();
    let dialog_cancel_clone = dialog.clone();
    cancel_btn.connect_clicked(move |_| {
        dialog_cancel_clone.close();
    });

    let copy_btn = Button::builder()
        .name("copy-button")
        .label("Copy & Close")
        .build();

    let on_copy_rc = Rc::new(on_copy);
    let dialog_copy_clone = dialog.clone();
    let prompt_buffer_copy = prompt_buffer.clone();

    copy_btn.connect_clicked(move |_| {
        let result = get_buffer_text(&prompt_buffer_copy);

        let display = gtk4::prelude::WidgetExt::display(&dialog_copy_clone);
        display.clipboard().set_text(&result);

        on_copy_rc(&result);
        dialog_copy_clone.close();
    });

    button_box.append(&cancel_btn);
    button_box.append(&copy_btn);
    main_box.append(&button_box);

    content_area.append(&main_box);

    dialog.show();
    window_hints::present_centered_always_on_top(
        &dialog,
        VARIABLE_DIALOG_WIDTH,
        VARIABLE_DIALOG_HEIGHT,
        first_input.as_ref(),
    );
}

fn attach_prompt_update_handlers(
    template: String,
    inputs: Rc<Vec<(String, VarType, gtk4::Widget)>>,
    prompt_buffer: &TextBuffer,
) {
    let template = Rc::new(template);
    for (_, var_type, widget) in inputs.iter() {
        match var_type {
            VarType::Text => {
                if let Some(entry) = widget.downcast_ref::<Entry>() {
                    let template = Rc::clone(&template);
                    let inputs = Rc::clone(&inputs);
                    let prompt_buffer = prompt_buffer.clone();
                    entry.connect_changed(move |_| {
                        set_prompt_editor_text(&template, &inputs, &prompt_buffer);
                    });
                }
            }
            VarType::Number => {
                if let Some(spin) = widget.downcast_ref::<SpinButton>() {
                    let template = Rc::clone(&template);
                    let inputs = Rc::clone(&inputs);
                    let prompt_buffer = prompt_buffer.clone();
                    spin.connect_value_changed(move |_| {
                        set_prompt_editor_text(&template, &inputs, &prompt_buffer);
                    });
                }
            }
            VarType::Option(_) => {
                if let Some(combo) = widget.downcast_ref::<ComboBoxText>() {
                    let template = Rc::clone(&template);
                    let inputs = Rc::clone(&inputs);
                    let prompt_buffer = prompt_buffer.clone();
                    combo.connect_changed(move |_| {
                        set_prompt_editor_text(&template, &inputs, &prompt_buffer);
                    });
                }
            }
            VarType::Multiline => {
                if let Some(text_view) = widget.downcast_ref::<TextView>() {
                    let variable_buffer = text_view.buffer();
                    let template = Rc::clone(&template);
                    let inputs = Rc::clone(&inputs);
                    let prompt_buffer = prompt_buffer.clone();
                    variable_buffer.connect_changed(move |_| {
                        set_prompt_editor_text(&template, &inputs, &prompt_buffer);
                    });
                }
            }
        }
    }
}

fn set_prompt_editor_text(
    template: &str,
    inputs: &[(String, VarType, gtk4::Widget)],
    prompt_buffer: &TextBuffer,
) {
    let mut values: Vec<(&str, String)> = Vec::with_capacity(inputs.len());
    for (name, var_type, widget) in inputs {
        values.push((name.as_str(), get_widget_value(widget, var_type)));
    }
    let result = interpolate(
        template,
        &values
            .iter()
            .map(|(name, value)| (*name, value.as_str()))
            .collect::<Vec<_>>(),
    );
    prompt_buffer.set_text(&result);
}

fn get_buffer_text(buffer: &TextBuffer) -> String {
    let (start, end) = buffer.bounds();
    buffer.text(&start, &end, false).to_string()
}

/// Extract the current value from a widget based on its type.
fn get_widget_value(widget: &gtk4::Widget, var_type: &crate::prompt_parser::VarType) -> String {
    match var_type {
        crate::prompt_parser::VarType::Text => {
            if let Some(entry) = widget.downcast_ref::<Entry>() {
                entry.text().to_string()
            } else {
                String::new()
            }
        }
        crate::prompt_parser::VarType::Number => {
            if let Some(spin) = widget.downcast_ref::<SpinButton>() {
                spin.value().to_string()
            } else {
                String::new()
            }
        }
        crate::prompt_parser::VarType::Option(_) => {
            if let Some(combo) = widget.downcast_ref::<ComboBoxText>() {
                combo
                    .active_text()
                    .map(|s| s.to_string())
                    .unwrap_or_default()
            } else {
                String::new()
            }
        }
        crate::prompt_parser::VarType::Multiline => {
            if let Some(text_view) = widget.downcast_ref::<TextView>() {
                let buffer = text_view.buffer();
                let (start, end) = buffer.bounds();
                buffer.text(&start, &end, false).to_string()
            } else {
                String::new()
            }
        }
    }
}
