import { useCallback, useState } from "react";
import { api } from "../api/commands";
import type { Prompt, SavePromptPayload } from "../types";

export function usePrompts() {
  const [prompts, setPrompts] = useState<Prompt[]>([]);

  const loadPrompts = useCallback(async () => {
    try {
      const list = await api.listPrompts();
      setPrompts(list);
    } catch {
      // silent
    }
  }, []);

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

  const deletePrompt = useCallback(async (id: number, name: string) => {
    await api.deletePrompt({ id, name });
    removePrompt(id);
  }, [removePrompt]);

  return {
    prompts,
    setPrompts,
    loadPrompts,
    patchPrompt,
    savePrompt,
    deletePrompt,
  };
}
