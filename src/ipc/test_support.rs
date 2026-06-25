use std::cell::RefCell;
use std::rc::Rc;

use super::effects::DesktopEffects;
use super::notifications::AppNotification;
use super::IpcBackend;
use crate::update::UpdateInfo;

#[derive(Default)]
pub struct FakeEffects {
    pub notifications: Rc<RefCell<Vec<AppNotification>>>,
    pub copied: Rc<RefCell<Vec<String>>>,
    pub copy_ok: bool,
}

impl DesktopEffects for FakeEffects {
    fn notify(&self, title: &str, body: &str) {
        let id = format!("fake-{}", self.notifications.borrow().len());
        self.notifications.borrow_mut().push(AppNotification {
            id,
            title: title.to_string(),
            body: body.to_string(),
            ephemeral: true,
            auto_close_ms: Some(3000),
            action_id: None,
            action_label: None,
            action_payload: None,
        });
    }

    fn notify_update_available(&self, info: &UpdateInfo) {
        let id = format!("fake-{}", self.notifications.borrow().len());
        self.notifications.borrow_mut().push(AppNotification {
            id,
            title: "Update available".to_string(),
            body: format!(
                "Promptly can be upgraded from {} to {}",
                info.current, info.latest
            ),
            ephemeral: false,
            auto_close_ms: None,
            action_id: Some("showUpdate".to_string()),
            action_label: Some("View update".to_string()),
            action_payload: None,
        });
    }

    fn notify_up_to_date(&self, latest: &str) {
        let id = format!("fake-{}", self.notifications.borrow().len());
        self.notifications.borrow_mut().push(AppNotification {
            id,
            title: "Promptly is up to date".to_string(),
            body: format!("Promptly is already at the latest version: {latest}"),
            ephemeral: true,
            auto_close_ms: Some(3000),
            action_id: None,
            action_label: None,
            action_payload: None,
        });
    }

    fn notify_update_check_failed(&self, message: &str) {
        let id = format!("fake-{}", self.notifications.borrow().len());
        self.notifications.borrow_mut().push(AppNotification {
            id,
            title: "Update check failed".to_string(),
            body: message.to_string(),
            ephemeral: true,
            auto_close_ms: Some(3000),
            action_id: None,
            action_label: None,
            action_payload: None,
        });
    }

    fn copy_text(&self, text: &str) -> anyhow::Result<()> {
        self.copied.borrow_mut().push(text.to_string());
        if self.copy_ok {
            Ok(())
        } else {
            anyhow::bail!("clipboard unavailable")
        }
    }
}

pub fn test_backend() -> (IpcBackend, tempfile::NamedTempFile) {
    let file = tempfile::NamedTempFile::new().unwrap();
    let backend = IpcBackend::new_for_test(file.path().to_path_buf()).unwrap();
    (backend, file)
}

pub fn handle(backend: &IpcBackend, effects: &FakeEffects, raw: &str) -> serde_json::Value {
    let handled = backend.handle(raw, effects);
    serde_json::from_str::<serde_json::Value>(&handled.response_json).unwrap()
}

pub fn handle_raw(
    backend: &IpcBackend,
    effects: &FakeEffects,
    raw: &str,
) -> super::types::HandledIpc {
    backend.handle(raw, effects)
}
