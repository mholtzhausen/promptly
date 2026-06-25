use std::cell::RefCell;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::update::UpdateInfo;

static NOTIFICATION_COUNTER: AtomicU64 = AtomicU64::new(0);

fn next_notification_id() -> String {
    let n = NOTIFICATION_COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("n{n}")
}

/// In-app notification pushed to the React footer.
#[derive(Debug, Clone, serde::Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AppNotification {
    pub id: String,
    pub title: String,
    pub body: String,
    pub ephemeral: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auto_close_ms: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action_label: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action_payload: Option<serde_json::Value>,
}

/// Collects in-app notifications during IPC handling or update checks.
pub struct NotificationCollector {
    notifications: RefCell<Vec<AppNotification>>,
    ephemeral_ms: u64,
}

impl NotificationCollector {
    pub fn new(ephemeral_ms: u64) -> Self {
        Self {
            notifications: RefCell::new(Vec::new()),
            ephemeral_ms,
        }
    }

    pub fn take(self) -> Vec<AppNotification> {
        self.notifications.into_inner()
    }

    fn push(&self, notification: AppNotification) {
        self.notifications.borrow_mut().push(notification);
    }

    fn ephemeral(&self, title: impl Into<String>, body: impl Into<String>) -> AppNotification {
        AppNotification {
            id: next_notification_id(),
            title: title.into(),
            body: body.into(),
            ephemeral: true,
            auto_close_ms: Some(self.ephemeral_ms),
            action_id: None,
            action_label: None,
            action_payload: None,
        }
    }
}

impl super::effects::DesktopEffects for NotificationCollector {
    fn notify(&self, title: &str, body: &str) {
        self.push(self.ephemeral(title, body));
    }

    fn notify_update_available(&self, info: &UpdateInfo) {
        let body = format!(
            "Promptly can be upgraded from {} to {}",
            info.current, info.latest
        );
        let payload = serde_json::json!({
            "currentVersion": info.current,
            "latestVersion": info.latest,
            "changelog": info.changelog,
        });
        self.push(AppNotification {
            id: next_notification_id(),
            title: "Update available".to_string(),
            body,
            ephemeral: false,
            auto_close_ms: None,
            action_id: Some("showUpdate".to_string()),
            action_label: Some("View update".to_string()),
            action_payload: Some(payload),
        });
    }

    fn notify_up_to_date(&self, latest: &str) {
        self.push(self.ephemeral(
            "Promptly is up to date",
            format!("Promptly is already at the latest version: {latest}"),
        ));
    }

    fn notify_update_check_failed(&self, message: &str) {
        self.push(self.ephemeral("Update check failed", message));
    }

    fn copy_text(&self, text: &str) -> anyhow::Result<()> {
        let mut clipboard = arboard::Clipboard::new()?;
        clipboard.set_text(text.to_string())?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ipc::effects::DesktopEffects;

    #[test]
    fn ephemeral_notifications_include_auto_close_ms() {
        let collector = NotificationCollector::new(3000);
        collector.notify("Title", "Body");
        let notifs = collector.take();
        assert_eq!(notifs.len(), 1);
        assert!(notifs[0].ephemeral);
        assert_eq!(notifs[0].auto_close_ms, Some(3000));
    }

    #[test]
    fn update_available_is_persistent_with_action() {
        let collector = NotificationCollector::new(3000);
        collector.notify_update_available(&UpdateInfo {
            current: "0.1.0".into(),
            latest: "0.2.0".into(),
            latest_tag: "v0.2.0".into(),
            changelog: "Changes".into(),
        });
        let notifs = collector.take();
        assert_eq!(notifs.len(), 1);
        assert!(!notifs[0].ephemeral);
        assert_eq!(notifs[0].action_id.as_deref(), Some("showUpdate"));
        assert!(notifs[0].action_payload.is_some());
    }
}
