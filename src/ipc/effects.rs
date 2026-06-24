/// Desktop side-effects the IPC layer needs: notifications and clipboard writes.
/// Implemented by `RealDesktopEffects` in production and fakes in tests.
pub trait DesktopEffects {
    fn notify(&self, summary: &str, body: &str);
    fn copy_text(&self, text: &str) -> anyhow::Result<()>;
}

/// Real desktop effects using `notify-rust` and `arboard`.
pub struct RealDesktopEffects;

impl DesktopEffects for RealDesktopEffects {
    fn notify(&self, summary: &str, body: &str) {
        use notify_rust::Notification;
        if let Err(e) = Notification::new().summary(summary).body(body).show() {
            log::error!("Failed to show notification: {}", e);
        }
    }

    fn copy_text(&self, text: &str) -> anyhow::Result<()> {
        let mut clipboard = arboard::Clipboard::new()?;
        clipboard.set_text(text.to_string())?;
        Ok(())
    }
}
