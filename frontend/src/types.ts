export interface Prompt {
  id: number;
  name: string;
  description: string;
  content: string;
}

export interface VariableDto {
  name: string;
  kind: "text" | "number" | "option" | "multiline";
  defaultValue: string;
  description: string;
  options: string[];
}

export interface SavePromptPayload {
  id: number | null;
  name: string;
  description: string;
  content: string;
}

export interface DeletePromptPayload {
  id: number;
  name: string;
}

export interface TemplatePayload {
  content: string;
}

export interface VariableValue {
  name: string;
  value: string;
}

export interface InterpolatePayload {
  template: string;
  values: VariableValue[];
}

export type CopyMessageKind = "noVariables" | "variables";

export interface CopyPromptPayload {
  text: string;
  promptName: string;
  promptId: number | null;
  values: VariableValue[];
  messageKind: CopyMessageKind;
  skipHistory?: boolean;
}

export interface CopyPromptResult {
  copied: boolean;
  historyInserted: boolean;
  historyCount: number;
}

export interface HistoryListItem {
  id: number;
  title: string;
  createdAt: number;
}

export interface HistoryListResult {
  entries: HistoryListItem[];
  totalCount: number;
}

export interface HistoryEntry {
  id: number;
  title: string;
  content: string;
  variables: VariableValue[];
  promptId: number | null;
  promptName: string;
  createdAt: number;
}

export interface HistoryIdPayload {
  id: number;
}

export interface UpdateHistoryEntryPayload {
  id: number;
  content: string;
}

export interface PruneHistoryPayload {
  keep: number;
}

export interface SavePromptResult {
  saved: boolean;
  prompt: Prompt | null;
}

export interface IpcRequest {
  id: string;
  command: string;
  payload?: unknown;
}

export type IpcResponse<T> =
  | { id: string; ok: true; data: T }
  | { id: string; ok: false; error: string };
