import type { UpdateDialogPayload } from "../components/UpdateView";

export interface AppNotification {
  id: string;
  title: string;
  body: string;
  ephemeral: boolean;
  autoCloseMs?: number;
  actionId?: string;
  actionLabel?: string;
  actionPayload?: UpdateDialogPayload;
}
