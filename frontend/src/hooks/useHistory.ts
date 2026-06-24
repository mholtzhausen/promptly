import { useCallback, useState } from "react";
import { api } from "../api/commands";
import type { HistoryListItem } from "../types";

export function useHistory(onError?: (message: string) => void) {
  const [historyEntries, setHistoryEntries] = useState<HistoryListItem[]>([]);
  const [historyTotalCount, setHistoryTotalCount] = useState(0);

  const reportError = useCallback(
    (err: unknown, fallback: string) => {
      const msg = err instanceof Error && err.message ? err.message : fallback;
      onError?.(msg);
    },
    [onError],
  );

  const loadHistory = useCallback(async () => {
    try {
      const result = await api.listHistory();
      setHistoryEntries(result.entries);
      setHistoryTotalCount(result.totalCount);
    } catch (err) {
      reportError(err, "Could not load history.");
    }
  }, [reportError]);

  const deleteHistoryItem = useCallback(
    async (id: number) => {
      try {
        await api.deleteHistoryEntry(id);
        await loadHistory();
      } catch (err) {
        reportError(err, "Could not delete history entry.");
      }
    },
    [loadHistory, reportError],
  );

  const pruneHistoryKeep = useCallback(
    async (keep: number) => {
      try {
        await api.pruneHistory(keep);
        await loadHistory();
      } catch (err) {
        reportError(err, "Could not prune history.");
      }
    },
    [loadHistory, reportError],
  );

  const getHistoryEntry = useCallback(
    (id: number) => api.getHistoryEntry(id),
    [],
  );

  const updateHistoryContent = useCallback(
    async (id: number, content: string) => {
      try {
        await api.updateHistoryEntry({ id, content });
      } catch (err) {
        reportError(err, "Could not update history entry.");
        throw err;
      }
    },
    [reportError],
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
