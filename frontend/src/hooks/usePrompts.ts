import { useCallback, useState } from "react";
import { api } from "../api/commands";
import type { Prompt, SavePromptPayload } from "../types";

export function usePrompts(onError?: (message: string) => void) {
  const [prompts, setPrompts] = useState<Prompt[]>([]);

  const reportError = useCallback(
    (err: unknown, fallback: string) => {
      const msg = err instanceof Error && err.message ? err.message : fallback;
      onError?.(msg);
    },
    [onError],
  );

  const loadPrompts = useCallback(async () => {
    try {
      const list = await api.listPrompts();
      setPrompts(list);
    } catch (err) {
      reportError(err, "Could not load prompts.");
    }
  }, [reportError]);

  const patchPrompt = useCallback((prompt: Prompt) => {
    setPrompts((prev) => {
      const idx = prev.findIndex((p) => p.id === prompt.id);
      if (idx >= 0) {
        const next = [...prev];
        next[idx] = prompt;
        return next.sort((a, b) => a.name.localeCompare(b.name));
      }
      return [...prev, prompt].sort((a, b) => a.name.localeCompare(b.name));
    });
  }, []);

  const removePrompt = useCallback((id: number) => {
    setPrompts((prev) => prev.filter((p) => p.id !== id));
  }, []);

  const savePrompt = useCallback(
    async (payload: SavePromptPayload) => api.savePrompt(payload),
    [],
  );

  const deletePrompt = useCallback(
    async (id: number, name: string) => {
      try {
        await api.deletePrompt({ id, name });
        removePrompt(id);
      } catch (err) {
        reportError(err, "Could not delete prompt.");
        throw err;
      }
    },
    [removePrompt, reportError],
  );

  return {
    prompts,
    setPrompts,
    loadPrompts,
    patchPrompt,
    savePrompt,
    deletePrompt,
  };
}
