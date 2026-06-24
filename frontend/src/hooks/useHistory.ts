import { useCallback, useState } from "react";
import { api } from "../api/commands";
import type { HistoryListItem } from "../types";

export function useHistory() {
  const [historyEntries, setHistoryEntries] = useState<HistoryListItem[]>([]);
  const [historyTotalCount, setHistoryTotalCount] = useState(0);

  const loadHistory = useCallback(async () => {
    try {
      const result = await api.listHistory();
      setHistoryEntries(result.entries);
      setHistoryTotalCount(result.totalCount);
    } catch {
      // silent
    }
  }, []);

  const deleteHistoryItem = useCallback(
    async (id: number) => {
      await api.deleteHistoryEntry(id);
      await loadHistory();
    },
    [loadHistory],
  );

  const pruneHistoryKeep = useCallback(
    async (keep: number) => {
      await api.pruneHistory(keep);
      await loadHistory();
    },
    [loadHistory],
  );

  const getHistoryEntry = useCallback(
    (id: number) => api.getHistoryEntry(id),
    [],
  );

  const updateHistoryContent = useCallback(
    async (id: number, content: string) => {
      await api.updateHistoryEntry({ id, content });
    },
    [],
  );

  return {
    historyEntries,
    historyTotalCount,
    loadHistory,
    deleteHistoryItem,
    pruneHistoryKeep,
    getHistoryEntry,
    updateHistoryContent,
  };
}
