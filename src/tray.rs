//! System tray icon using `ksni` crate — works on X11 and Wayland without GTK3 conflicts.

use anyhow::Result;
use ksni::blocking::TrayMethods;
use ksni::{menu::*, MenuItem, Tray};
use std::sync::mpsc::Sender;

struct PromptlyTray {
    tx: Sender<()>,
}

impl Tray for PromptlyTray {
    fn id(&self) -> String {
        "promptly".into()
    }

    fn icon_name(&self) -> String {
        "edit-paste".into() // Standard paste icon name
    }

    fn title(&self) -> String {
        "Promptly".into()
    }

    fn menu(&self) -> Vec<MenuItem<Self>> {
        let tx_show = self.tx.clone();
        vec![
            StandardItem {
                label: "Show Prompt Manager".into(),
                activate: Box::new(move |_| {
                    let _ = tx_show.send(());
                }),
                ..Default::default()
            }
            .into(),
            MenuItem::Separator,
            StandardItem {
                label: "Quit".into(),
                activate: Box::new(|_| {
                    std::process::exit(0);
                }),
                ..Default::default()
            }
            .into(),
        ]
    }
}

pub struct TrayState {
    // We keep the handle alive so the tray service runs
    _handle: ksni::blocking::Handle<PromptlyTray>,
}

impl TrayState {
    /// Build and show a system tray icon using `ksni`.
    pub fn build(tx: Sender<()>) -> Result<Self> {
        let tray = PromptlyTray { tx };
        tray.spawn()
            .map_err(|e| {
                anyhow::anyhow!(
                    "{}. Install a tray StatusNotifierHost (e.g. gnome-shell-extension-appindicator, \
                     KDE's statusnotifierwatcher, or another DBus status notifier provider)",
                    e
                )
            })
            .map(|handle| Self { _handle: handle })
    }
}