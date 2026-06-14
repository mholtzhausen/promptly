//! Variable input dialog with type-aware widgets and clipboard copy.

use gtk4::prelude::*;
use gtk4::{
    Align, Box as GtkBox, Button, ComboBoxText, Dialog, Entry, Label, Orientation, SpinButton,
    TextBuffer, TextView, Window,
};
use std::rc::Rc;

use crate::config::CSS;
use crate::prompt_parser::{interpolate, Variable};

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
        .default_width(400)
        .build();

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
    let main_box = GtkBox::new(Orientation::Vertical, 12);
    main_box.set_margin_start(16);
    main_box.set_margin_end(16);
    main_box.set_margin_top(16);
    main_box.set_margin_bottom(16);

    // Template preview (read-only)
    let template_label = Label::builder()
        .label(template)
        .wrap(true)
        .selectable(true)
        .halign(Align::Start)
        .build();
    main_box.append(&template_label);

    // Variable input widgets
    let mut inputs = Vec::new();

    for var in variables {
        let var_box = GtkBox::new(Orientation::Vertical, 4);
        var_box.set_margin_start(4);

        // Label + description
        let label_text = if !var.description.is_empty() {
            format!("{} ({})", var.name, var.description)
        } else {
            var.name.clone()
        };
        let var_label = Label::builder()
            .label(&label_text)
            .halign(Align::Start)
            .build();

        // Input widget based on type
        let input_widget: gtk4::Widget = match &var.var_type {
            crate::prompt_parser::VarType::Text => {
                let entry = Entry::builder()
                    .name("variable-entry")
                    .text(&var.default_value)
                    .build();
                entry.set_hexpand(true);
                entry.into()
            }
            crate::prompt_parser::VarType::Number => {
                let adj = gtk4::Adjustment::builder()
                    .value(var.default_value.parse::<f64>().unwrap_or(0.0))
                    .lower(f64::MIN)
                    .upper(f64::MAX)
                    .step_increment(1.0)
                    .build();
                let spin = SpinButton::builder()
                    .adjustment(&adj)
                    .name("variable-entry")
                    .build();
                spin.set_hexpand(true);
                spin.into()
            }
            crate::prompt_parser::VarType::Option(options) => {
                let combo = ComboBoxText::builder().name("variable-entry").build();
                for opt in options {
                    combo.append_text(opt);
                }
                // Select default if it matches an option
                if !var.default_value.is_empty() && options.contains(&var.default_value) {
                    combo.set_active_id(Some(&var.default_value));
                } else if !options.is_empty() {
                    ComboBoxExtManual::set_active(&combo, Some(0));
                }
                combo.set_hexpand(true);
                combo.into()
            }
            crate::prompt_parser::VarType::Multiline => {
                let buffer = TextBuffer::builder().text(&var.default_value).build();
                let text_view = TextView::builder()
                    .buffer(&buffer)
                    .name("variable-textview")
                    .wrap_mode(gtk4::WrapMode::WordChar)
                    .vexpand(true)
                    .build();
                text_view.into()
            }
        };

        var_box.append(&var_label);
        var_box.append(&input_widget);
        main_box.append(&var_box);

        inputs.push((var.name.clone(), var.var_type.clone(), input_widget));
    }

    // Buttons row
    let button_box = GtkBox::new(Orientation::Horizontal, 8);
    button_box.set_halign(Align::End);
    button_box.set_margin_top(16);

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
    let template_string = template.to_string();

    copy_btn.connect_clicked(move |_| {
        // Collect values from all input widgets
        let mut values: Vec<(&str, String)> = Vec::new();
        for (name, var_type, widget) in &inputs {
            let value = get_widget_value(widget, var_type);
            values.push((name.as_str(), value));
        }

        // Interpolate the template
        let result = interpolate(
            &template_string,
            &values
                .iter()
                .map(|(n, v)| (*n, v.as_str()))
                .collect::<Vec<_>>(),
        );

        // Copy to clipboard
        match arboard::Clipboard::new() {
            Ok(mut clipboard) => {
                if let Err(e) = clipboard.set_text(&result) {
                    log::error!("Failed to copy to clipboard: {}", e);
                }
            }
            Err(e) => {
                log::error!("Failed to create clipboard: {}", e);
            }
        }

        on_copy_rc(&result);
        dialog_copy_clone.close();
    });

    button_box.append(&cancel_btn);
    button_box.append(&copy_btn);
    main_box.append(&button_box);

    content_area.append(&main_box);

    // Show the dialog
    dialog.show();
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
