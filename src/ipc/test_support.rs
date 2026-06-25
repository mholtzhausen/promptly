use std::cell::RefCell;
use std::rc::Rc;

use super::effects::DesktopEffects;
use super::IpcBackend;

#[derive(Default)]
pub struct FakeEffects {
    pub notifications: Rc<RefCell<Vec<(String, String)>>>,
    pub copied: Rc<RefCell<Vec<String>>>,
    pub copy_ok: bool,
    pub update_actions: Rc<RefCell<Vec<(String, String)>>>,
    pub up_to_date: Rc<RefCell<Vec<String>>>,
    pub update_check_failed: Rc<RefCell<Vec<String>>>,
}

impl DesktopEffects for FakeEffects {
    fn notify(&self, summary: &str, body: &str) {
        self.notifications
            .borrow_mut()
            .push((summary.to_string(), body.to_string()));
    }

    fn notify_update_available(
        &self,
        current: &str,
        latest: &str,
        on_action: Box<dyn FnOnce() + Send>,
    ) {
        self.update_actions
            .borrow_mut()
            .push((current.to_string(), latest.to_string()));
        on_action();
    }

    fn notify_up_to_date(&self, latest: &str) {
        self.up_to_date.borrow_mut().push(latest.to_string());
    }

    fn notify_update_check_failed(&self, message: &str) {
        self.update_check_failed.borrow_mut().push(message.to_string());
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
    let backend = IpcBackend::new(file.path().to_path_buf()).unwrap();
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
