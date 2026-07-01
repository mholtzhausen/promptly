import type {
  CopyPromptPayload,
  CopyMessageKind,
  CopySettings,
  DeletePromptPayload,
  HistoryEntry,
  HistoryIdPayload,
  HistoryListResult,
  InterpolatePayload,
  Prompt,
  PruneHistoryPayload,
  SavePromptPayload,
  SavePromptResult,
  UpdateHistoryEntryPayload,
  VariableDto,
  VariableValue,
} from "../types";
import { request } from "./ipc";

export const api = {
  listPrompts: () => request<Prompt[]>("listPrompts"),

  savePrompt: (payload: SavePromptPayload) =>
    request<SavePromptResult>("savePrompt", payload),

  deletePrompt: (payload: DeletePromptPayload) =>
    request<boolean>("deletePrompt", payload),

  variablesForTemplate: (content: string) =>
    request<VariableDto[]>("variablesForTemplate", { content }),

  interpolate: (payload: InterpolatePayload) =>
    request<string>("interpolate", payload),

  copyPrompt: (payload: CopyPromptPayload) =>
    request<{ copied: boolean; historyInserted: boolean; historyCount: number }>(
      "copyPrompt",
      payload,
    ),

  listHistory: () => request<HistoryListResult>("listHistory"),

  getHistoryEntry: (id: number) =>
    request<HistoryEntry | null>("getHistoryEntry", { id } satisfies HistoryIdPayload),

  updateHistoryEntry: (payload: UpdateHistoryEntryPayload) =>
    request<boolean>("updateHistoryEntry", payload),

  deleteHistoryEntry: (id: number) =>
    request<boolean>("deleteHistoryEntry", { id } satisfies HistoryIdPayload),

  pruneHistory: (keep: number) =>
    request<boolean>("pruneHistory", { keep } satisfies PruneHistoryPayload),

  setWindowTitle: (title: string) =>
    request<boolean>("setWindowTitle", { title }),

  hideWindow: () => request<boolean>("hideWindow"),

  quit: () => request<boolean>("quit"),

  runUpdate: () => request<boolean>("runUpdate"),

  getAppInfo: () =>
    request<{ version: string; description: string; features: string[] }>(
      "getAppInfo",
    ),

  getCopySettings: () => request<CopySettings>("getCopySettings"),

  setLastCopyTarget: (name: string) =>
    request<boolean>("setLastCopyTarget", { name }),

  openCopyTarget: (name: string) =>
    request<boolean>("openCopyTarget", { name }),

  getAppSettings: () =>
    request<import("../types").AppSettings>("getAppSettings"),

  saveAppSettings: (payload: import("../types").SaveAppSettingsPayload) =>
    request<import("../types").AppSettings>("saveAppSettings", payload),

  openSettingsWindow: () => request<boolean>("openSettingsWindow"),

  closeSettingsWindow: () => request<boolean>("closeSettingsWindow"),
};

export type CopyPromptArgs = {
  text: string;
  promptName: string;
  messageKind: CopyMessageKind;
  promptId: number | null;
  values: VariableValue[];
  skipHistory?: boolean;
};

export async function copyPromptToClipboard(args: CopyPromptArgs): Promise<void> {
  await api.copyPrompt({
    text: args.text,
    promptName: args.promptName,
    promptId: args.promptId,
    values: args.values,
    messageKind: args.messageKind,
    skipHistory: args.skipHistory,
  });
}
