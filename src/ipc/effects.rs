use crate::update::UpdateInfo;

/// Desktop side-effects the IPC layer needs: in-app notifications and clipboard writes.
pub trait DesktopEffects {
    fn notify(&self, title: &str, body: &str);
    fn notify_update_available(&self, info: &UpdateInfo);
    fn notify_up_to_date(&self, latest: &str);
    fn notify_update_check_failed(&self, message: &str);
    fn copy_text(&self, text: &str) -> anyhow::Result<()>;
}
