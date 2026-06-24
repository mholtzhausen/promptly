use std::cell::RefCell;
use std::rc::Rc;

use super::effects::DesktopEffects;
use super::IpcBackend;

#[derive(Default)]
pub struct FakeEffects {
    pub notifications: Rc<RefCell<Vec<(String, String)>>>,
    pub copied: Rc<RefCell<Vec<String>>>,
    pub copy_ok: bool,
}

impl DesktopEffects for FakeEffects {
    fn notify(&self, summary: &str, body: &str) {
        self.notifications
            .borrow_mut()
            .push((summary.to_string(), body.to_string()));
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
