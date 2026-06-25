import type { AppNotification } from "../types/notifications";

type NotificationFooterProps = {
  notifications: AppNotification[];
  onDismiss: (id: string) => void;
  onAction: (notification: AppNotification) => void;
};

export function NotificationFooter({
  notifications,
  onDismiss,
  onAction,
}: NotificationFooterProps) {
  if (notifications.length === 0) {
    return null;
  }

  return (
    <div
      className="notification-stack"
      role="region"
      aria-label="Notifications"
      aria-live="polite"
    >
      {notifications.map((notification) => (
        <div
          key={notification.id}
          className={
            "notification-item" + (notification.ephemeral ? " ephemeral" : " persistent")
          }
        >
          <div className="notification-content">
            <p className="notification-title">{notification.title}</p>
            <p className="notification-body">{notification.body}</p>
          </div>
          <div className="notification-actions">
            {notification.actionId && notification.actionLabel ? (
              <button
                type="button"
                className="notification-action-btn"
                onClick={() => onAction(notification)}
              >
                {notification.actionLabel}
              </button>
            ) : null}
            <button
              type="button"
              className="notification-dismiss-btn"
              aria-label="Dismiss notification"
              onClick={() => onDismiss(notification.id)}
            >
              ×
            </button>
          </div>
        </div>
      ))}
    </div>
  );
}
