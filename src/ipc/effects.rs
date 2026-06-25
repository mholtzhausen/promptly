/// Desktop side-effects the IPC layer needs: notifications and clipboard writes.
/// Implemented by `RealDesktopEffects` in production and fakes in tests.
pub trait DesktopEffects {
    fn notify(&self, summary: &str, body: &str);
    fn notify_update_available(
        &self,
        current: &str,
        latest: &str,
        on_action: Box<dyn FnOnce() + Send>,
    );
    fn notify_up_to_date(&self, latest: &str);
    fn notify_update_check_failed(&self, message: &str);
    fn copy_text(&self, text: &str) -> anyhow::Result<()>;
}

/// Real desktop effects using `notify-rust` and `arboard`.
pub struct RealDesktopEffects;

const APP_NAME: &str = "promptly";

impl DesktopEffects for RealDesktopEffects {
    fn notify(&self, summary: &str, body: &str) {
        use notify_rust::Notification;
        if let Err(e) = Notification::new()
            .appname(APP_NAME)
            .summary(summary)
            .body(body)
            .show()
        {
            log::error!("Failed to show notification: {}", e);
        }
    }

    fn notify_update_available(
        &self,
        current: &str,
        latest: &str,
        on_action: Box<dyn FnOnce() + Send>,
    ) {
        use notify_rust::Notification;

        let body = format!("Promptly can be upgraded from {current} to {latest}");
        // Show on the calling (main) thread so the notification reliably appears,
        // then wait for the action on a background thread since wait_for_action
        // blocks until the user interacts with or dismisses the notification.
        match Notification::new()
            .appname(APP_NAME)
            .summary("Update available")
            .body(&body)
            .action("update", "click here to update")
            .show()
        {
            Ok(handle) => {
                std::thread::spawn(move || {
                    handle.wait_for_action(move |action| {
                        if action == "update" {
                            on_action();
                        }
                    });
                });
            }
            Err(e) => log::error!("Failed to show update notification: {e}"),
        }
    }

    fn notify_up_to_date(&self, latest: &str) {
        use notify_rust::Notification;
        let body = format!("Promptly is already at the latest version: {latest}");
        if let Err(e) = Notification::new()
            .appname(APP_NAME)
            .summary("Promptly is up to date")
            .body(&body)
            .show()
        {
            log::error!("Failed to show up-to-date notification: {e}");
        }
    }

    fn notify_update_check_failed(&self, message: &str) {
        use notify_rust::Notification;
        if let Err(e) = Notification::new()
            .appname(APP_NAME)
            .summary("Update check failed")
            .body(message)
            .show()
        {
            log::error!("Failed to show update-check-failed notification: {e}");
        }
    }

    fn copy_text(&self, text: &str) -> anyhow::Result<()> {
        let mut clipboard = arboard::Clipboard::new()?;
        clipboard.set_text(text.to_string())?;
        Ok(())
    }
}
