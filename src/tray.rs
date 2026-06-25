//! System tray icon using `ksni` crate — works on X11 and Wayland without GTK3 conflicts.

use anyhow::Result;
use ksni::blocking::TrayMethods;
use ksni::{menu::*, MenuItem, Tray};
use std::sync::mpsc::Sender;

/// Actions delivered from the tray menu to the main event loop bridge.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrayAction {
    ToggleWindow,
    CheckForUpdates,
    ShowAbout,
}

struct PromptlyTray {
    tx: Sender<TrayAction>,
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
        let tx_updates = self.tx.clone();
        let tx_about = self.tx.clone();
        vec![
            StandardItem {
                label: "Show Prompt Manager".into(),
                activate: Box::new(move |_| {
                    let _ = tx_show.send(TrayAction::ToggleWindow);
                }),
                ..Default::default()
            }
            .into(),
            StandardItem {
                label: "Check for Updates".into(),
                activate: Box::new(move |_| {
                    let _ = tx_updates.send(TrayAction::CheckForUpdates);
                }),
                ..Default::default()
            }
            .into(),
            StandardItem {
                label: "About".into(),
                activate: Box::new(move |_| {
                    let _ = tx_about.send(TrayAction::ShowAbout);
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
    pub fn build(tx: Sender<TrayAction>) -> Result<Self> {
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
