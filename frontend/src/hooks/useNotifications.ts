import { useCallback, useEffect, useRef, useState } from "react";
import type { UpdateDialogPayload } from "../components/UpdateView";
import type { AppNotification } from "../types/notifications";

type UseNotificationsOptions = {
  onShowUpdateDialog: (payload: UpdateDialogPayload) => void;
};

export function useNotifications({ onShowUpdateDialog }: UseNotificationsOptions) {
  const [notifications, setNotifications] = useState<AppNotification[]>([]);
  const windowVisibleRef = useRef(false);
  const timersRef = useRef<Map<string, number>>(new Map());

  const clearTimer = useCallback((id: string) => {
    const timer = timersRef.current.get(id);
    if (timer !== undefined) {
      window.clearTimeout(timer);
      timersRef.current.delete(id);
    }
  }, []);

  const dismissNotification = useCallback(
    (id: string) => {
      clearTimer(id);
      setNotifications((prev) => prev.filter((n) => n.id !== id));
    },
    [clearTimer],
  );

  const scheduleAutoClose = useCallback(
    (notification: AppNotification) => {
      if (!notification.ephemeral) return;
      const ms = notification.autoCloseMs ?? 3000;
      clearTimer(notification.id);
      const timer = window.setTimeout(() => {
        timersRef.current.delete(notification.id);
        setNotifications((prev) => prev.filter((n) => n.id !== notification.id));
      }, ms);
      timersRef.current.set(notification.id, timer);
    },
    [clearTimer],
  );

  const startEphemeralTimers = useCallback(() => {
    setNotifications((prev) => {
      for (const notification of prev) {
        if (notification.ephemeral && !timersRef.current.has(notification.id)) {
          scheduleAutoClose(notification);
        }
      }
      return prev;
    });
  }, [scheduleAutoClose]);

  const clearAllTimers = useCallback(() => {
    for (const id of timersRef.current.keys()) {
      clearTimer(id);
    }
  }, [clearTimer]);

  const pushNotifications = useCallback(
    (incoming: AppNotification[]) => {
      if (incoming.length === 0) return;
      setNotifications((prev) => [...prev, ...incoming]);
      if (windowVisibleRef.current) {
        for (const notification of incoming) {
          if (notification.ephemeral) {
            scheduleAutoClose(notification);
          }
        }
      }
    },
    [scheduleAutoClose],
  );

  const onWindowVisible = useCallback(() => {
    windowVisibleRef.current = true;
    startEphemeralTimers();
  }, [startEphemeralTimers]);

  const onWindowHidden = useCallback(() => {
    windowVisibleRef.current = false;
    clearAllTimers();
  }, [clearAllTimers]);

  const runNotificationAction = useCallback(
    (notification: AppNotification) => {
      if (notification.actionId === "showUpdate" && notification.actionPayload) {
        onShowUpdateDialog(notification.actionPayload);
      }
      dismissNotification(notification.id);
    },
    [dismissNotification, onShowUpdateDialog],
  );

  useEffect(() => {
    window.__promptlyPushNotifications = pushNotifications;
    window.__promptlyOnWindowVisible = onWindowVisible;
    window.__promptlyOnWindowHidden = onWindowHidden;
    return () => {
      window.__promptlyPushNotifications = () => {};
      window.__promptlyOnWindowVisible = () => {};
      window.__promptlyOnWindowHidden = () => {};
      clearAllTimers();
    };
  }, [pushNotifications, onWindowVisible, onWindowHidden, clearAllTimers]);

  return {
    notifications,
    dismissNotification,
    runNotificationAction,
  };
}
