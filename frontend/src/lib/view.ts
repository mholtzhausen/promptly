import type { Prompt, HistoryEntry } from "../types";

export const TITLE_PREFIX = "Promptly | ";

export type View =
  | "list"
  | "editor"
  | "delete"
  | "variables"
  | "history"
  | "historyDetail";

export const PRUNE_KEEP_OPTIONS = [10, 100, 500, 1000] as const;

export function windowTitleForView(
  view: View,
  ctx: {
    editingPrompt: Prompt | null;
    variablePrompt: Prompt | null;
    deletingPrompt: Prompt | null;
    historyDetail: HistoryEntry | null;
  },
): string {
  switch (view) {
    case "list":
      return `${TITLE_PREFIX}Find a prompt`;
    case "editor":
      return ctx.editingPrompt
        ? `${TITLE_PREFIX}Edit ${ctx.editingPrompt.name}`
        : `${TITLE_PREFIX}New prompt template`;
    case "variables":
      return ctx.variablePrompt
        ? `${TITLE_PREFIX}Fill out ${ctx.variablePrompt.name}`
        : `${TITLE_PREFIX}Find a prompt`;
    case "history":
      return `${TITLE_PREFIX}Copy history`;
    case "historyDetail":
      return ctx.historyDetail
        ? `${TITLE_PREFIX}View ${ctx.historyDetail.promptName}`
        : `${TITLE_PREFIX}Copy history`;
    case "delete":
      return ctx.deletingPrompt
        ? `${TITLE_PREFIX}Delete ${ctx.deletingPrompt.name}`
        : `${TITLE_PREFIX}Find a prompt`;
  }
}
