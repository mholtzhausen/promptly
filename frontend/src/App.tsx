import { useState, useEffect, useCallback, useRef } from "react";
import { api, copyPromptToClipboard } from "./api/commands";
import { useHostBridge } from "./bridge/host";
import { DeleteView } from "./components/DeleteView";
import { EditorView } from "./components/EditorView";
import { HistoryDetailView } from "./components/HistoryDetailView";
import { HistoryView } from "./components/HistoryView";
import { ListView } from "./components/ListView";
import { VariablesView } from "./components/VariablesView";
import { useInterpolatePreview } from "./hooks/useInterpolatePreview";
import {
  useAggressiveFocus,
  usePrintableKeyToInput,
  useScrollSelectedIntoView,
  useSelectedIndexBounds,
} from "./hooks/useListKeyboard";
import { useHistory } from "./hooks/useHistory";
import { usePrompts } from "./hooks/usePrompts";
import { filterHistory, filterPrompts } from "./lib/fuzzy";
import type { View } from "./lib/view";
import { windowTitleForView } from "./lib/view";
import type { HistoryEntry, HistoryListItem, Prompt, VariableDto, VariableValue } from "./types";
import "./styles.css";

export default function App() {
  const [view, setView] = useState<View>("list");
  const [query, setQuery] = useState("");
  const [selectedIndex, setSelectedIndex] = useState(0);

  const [editingPrompt, setEditingPrompt] = useState<Prompt | null>(null);
  const [editorContent, setEditorContent] = useState("");
  const [deletingPrompt, setDeletingPrompt] = useState<Prompt | null>(null);
  const [editorError, setEditorError] = useState<string | null>(null);

  const [variablePrompt, setVariablePrompt] = useState<Prompt | null>(null);
  const [variables, setVariables] = useState<VariableDto[]>([]);
  const [variableValues, setVariableValues] = useState<Record<string, string>>({});
  const [preview, setPreview] = useState("");

  const [historyQuery, setHistoryQuery] = useState("");
  const [historySelectedIndex, setHistorySelectedIndex] = useState(0);
  const [historyDetail, setHistoryDetail] = useState<HistoryEntry | null>(null);
  const [historyDetailContent, setHistoryDetailContent] = useState("");
  const [pruneMenuOpen, setPruneMenuOpen] = useState(false);
  const [statusError, setStatusError] = useState<string | null>(null);

  const reportStatusError = useCallback((message: string) => {
    setStatusError(message);
  }, []);

  const clearStatusError = useCallback(() => {
    setStatusError(null);
  }, []);

  const searchRef = useRef<HTMLInputElement>(null);
  const historySearchRef = useRef<HTMLInputElement>(null);
  const pruneMenuRef = useRef<HTMLDivElement>(null);
  const listRef = useRef<HTMLDivElement>(null);
  const historyListRef = useRef<HTMLDivElement>(null);
  const editorFormRef = useRef<HTMLFormElement>(null);

  const focusSearch = useAggressiveFocus(searchRef);
  const focusHistorySearch = useAggressiveFocus(historySearchRef);

  const {
    prompts,
    loadPrompts,
    patchPrompt,
    savePrompt,
    deletePrompt,
  } = usePrompts(reportStatusError);

  const {
    historyEntries,
    historyTotalCount,
    loadHistory,
    deleteHistoryItem,
    pruneHistoryKeep,
    getHistoryEntry,
    updateHistoryContent,
  } = useHistory(reportStatusError);

  const onShow = useCallback(() => {
    setView("list");
    setQuery("");
    setSelectedIndex(0);
    clearStatusError();
    void loadPrompts();
    focusSearch();
  }, [loadPrompts, focusSearch, clearStatusError]);

  useHostBridge({ onShow, focusSearch });

  useEffect(() => {
    void loadPrompts();
  }, [loadPrompts]);

  useEffect(() => {
    const title = windowTitleForView(view, {
      editingPrompt,
      variablePrompt,
      deletingPrompt,
      historyDetail,
    });
    void api.setWindowTitle(title).catch(() => {});
  }, [view, editingPrompt, variablePrompt, deletingPrompt, historyDetail]);

  useEffect(() => {
    const onKeyDown = (e: KeyboardEvent) => {
      if (e.key !== "Escape" || !e.ctrlKey || e.metaKey || e.altKey) return;
      e.preventDefault();
      e.stopPropagation();
      void api.quit();
    };
    document.addEventListener("keydown", onKeyDown, true);
    return () => document.removeEventListener("keydown", onKeyDown, true);
  }, []);

  useEffect(() => {
    if (view !== "list") return;
    focusSearch();
    window.addEventListener("focus", focusSearch);
    return () => window.removeEventListener("focus", focusSearch);
  }, [view, focusSearch]);

  usePrintableKeyToInput(view === "list", searchRef, (key) => {
    setQuery((prev) => prev + key);
    setSelectedIndex(0);
  });

  usePrintableKeyToInput(view === "history", historySearchRef, (key) => {
    setHistoryQuery((prev) => prev + key);
    setHistorySelectedIndex(0);
  });

  useEffect(() => {
    if (view !== "history") return;
    focusHistorySearch();
    window.addEventListener("focus", focusHistorySearch);
    return () => window.removeEventListener("focus", focusHistorySearch);
  }, [view, focusHistorySearch]);

  useEffect(() => {
    if (!pruneMenuOpen) return;
    const onMouseDown = (e: MouseEvent) => {
      if (
        pruneMenuRef.current &&
        !pruneMenuRef.current.contains(e.target as Node)
      ) {
        setPruneMenuOpen(false);
      }
    };
    document.addEventListener("mousedown", onMouseDown);
    return () => document.removeEventListener("mousedown", onMouseDown);
  }, [pruneMenuOpen]);

  const filtered = filterPrompts(prompts, query);
  const filteredHistory = filterHistory(historyEntries, historyQuery);

  useSelectedIndexBounds(filtered.length, selectedIndex, setSelectedIndex);
  useScrollSelectedIntoView(view === "list", listRef, selectedIndex);

  useSelectedIndexBounds(
    filteredHistory.length,
    historySelectedIndex,
    setHistorySelectedIndex,
  );
  useScrollSelectedIntoView(
    view === "history",
    historyListRef,
    historySelectedIndex,
  );

  const buildCopyValues = useCallback((): VariableValue[] => {
    return variables.map((v) => ({
      name: v.name,
      value: variableValues[v.name] ?? "",
    }));
  }, [variables, variableValues]);

  const selectPrompt = useCallback(async (prompt: Prompt) => {
    try {
      const vars = await api.variablesForTemplate(prompt.content);
      setVariablePrompt(prompt);
      setVariables(vars);
      const values: Record<string, string> = {};
      for (const v of vars) {
        values[v.name] = v.defaultValue;
      }
      setVariableValues(values);
      const interp = await api.interpolate({
        template: prompt.content,
        values: vars.map((v) => ({ name: v.name, value: values[v.name] })),
      });
      setPreview(interp);
      setView("variables");
    } catch (err) {
      reportStatusError(
        err instanceof Error && err.message
          ? err.message
          : "Could not open prompt variables.",
      );
    }
  }, [reportStatusError]);

  const openEdit = useCallback((p: Prompt) => {
    setEditingPrompt(p);
    setEditorContent(p.content);
    setEditorError(null);
    setView("editor");
  }, []);

  const handleListKey = useCallback(
    (e: React.KeyboardEvent) => {
      if (view !== "list") return;
      switch (e.key) {
        case "Escape":
          e.preventDefault();
          void api.hideWindow();
          break;
        case "ArrowDown": {
          e.preventDefault();
          const max = filtered.length;
          if (max > 0) setSelectedIndex((prev) => (prev + 1) % max);
          break;
        }
        case "ArrowUp": {
          e.preventDefault();
          const max = filtered.length;
          if (max > 0) setSelectedIndex((prev) => (prev + max - 1) % max);
          break;
        }
        case "Enter": {
          e.preventDefault();
          const prompt = filtered[selectedIndex];
          if (!prompt) return;
          if (e.ctrlKey || e.metaKey) {
            openEdit(prompt);
          } else {
            void selectPrompt(prompt);
          }
          break;
        }
      }
    },
    [view, filtered, selectedIndex, selectPrompt, openEdit],
  );

  const openHistory = useCallback(async () => {
    await loadHistory();
    setHistoryQuery("");
    setHistorySelectedIndex(0);
    setView("history");
  }, [loadHistory]);

  const closeHistory = useCallback(() => {
    setView("list");
    setHistoryQuery("");
    setHistorySelectedIndex(0);
    focusSearch();
  }, [focusSearch]);

  const openHistoryDetail = useCallback(
    async (item: HistoryListItem) => {
      try {
        const entry = await getHistoryEntry(item.id);
        if (!entry) return;
        setHistoryDetail(entry);
        setHistoryDetailContent(entry.content);
        setView("historyDetail");
      } catch (err) {
        reportStatusError(
          err instanceof Error && err.message
            ? err.message
            : "Could not load history entry.",
        );
      }
    },
    [getHistoryEntry, reportStatusError],
  );

  const closeHistoryDetail = useCallback(() => {
    setHistoryDetail(null);
    setHistoryDetailContent("");
    setView("history");
    focusHistorySearch();
  }, [focusHistorySearch]);

  const handleHistoryKey = useCallback(
    (e: React.KeyboardEvent) => {
      if (view !== "history") return;
      switch (e.key) {
        case "Escape":
          e.preventDefault();
          e.stopPropagation();
          closeHistory();
          break;
        case "ArrowDown": {
          e.preventDefault();
          const max = filteredHistory.length;
          if (max > 0) setHistorySelectedIndex((prev) => (prev + 1) % max);
          break;
        }
        case "ArrowUp": {
          e.preventDefault();
          const max = filteredHistory.length;
          if (max > 0)
            setHistorySelectedIndex((prev) => (prev + max - 1) % max);
          break;
        }
        case "Enter": {
          e.preventDefault();
          const entry = filteredHistory[historySelectedIndex];
          if (entry) void openHistoryDetail(entry);
          break;
        }
      }
    },
    [
      view,
      filteredHistory,
      historySelectedIndex,
      closeHistory,
      openHistoryDetail,
    ],
  );

  const openNew = useCallback(() => {
    setEditingPrompt(null);
    setEditorContent("");
    setEditorError(null);
    setView("editor");
  }, []);

  const closeEditor = useCallback(() => {
    setView("list");
    setEditingPrompt(null);
    setEditorContent("");
    setEditorError(null);
    focusSearch();
  }, [focusSearch]);

  const handleSave = useCallback(async () => {
    const form = editorFormRef.current;
    if (!form) return;

    setEditorError(null);
    const data = new FormData(form);
    const id = editingPrompt?.id ?? null;
    const name = ((data.get("name") as string) ?? "").trim();
    const description = ((data.get("description") as string) ?? "").trim();
    const content = editorContent;

    if (!name || !description || !content.trim()) {
      setEditorError("Name, description, and content are all required.");
      return;
    }

    try {
      const result = await savePrompt({ id, name, description, content });
      if (!result?.saved) {
        setEditorError("Could not save — check that all fields are filled in.");
        return;
      }
      if (result.prompt) {
        patchPrompt(result.prompt);
      }
    } catch (err) {
      const msg =
        err instanceof Error && err.message
          ? err.message
          : "Could not save the prompt. Please try again.";
      setEditorError(msg);
      return;
    }

    setView("list");
    setEditingPrompt(null);
    setEditorContent("");
    setEditorError(null);
  }, [editingPrompt, editorContent, savePrompt, patchPrompt]);

  const openDelete = useCallback((p: Prompt) => {
    setDeletingPrompt(p);
    setView("delete");
  }, []);

  const closeDelete = useCallback(() => {
    setView("list");
    setDeletingPrompt(null);
  }, []);

  const confirmDelete = useCallback(async () => {
    if (!deletingPrompt) return;
    try {
      await deletePrompt(deletingPrompt.id, deletingPrompt.name);
    } catch {
      // silent
    }
    setView("list");
    setDeletingPrompt(null);
  }, [deletingPrompt, deletePrompt]);

  const handleVariableInput = useCallback((name: string, value: string) => {
    setVariableValues((prev) => ({ ...prev, [name]: value }));
  }, []);

  useInterpolatePreview(
    variablePrompt?.content,
    variableValues,
    setPreview,
    reportStatusError,
  );

  const resetVariables = useCallback(() => {
    setVariablePrompt(null);
    setVariables([]);
    setVariableValues({});
    setPreview("");
  }, []);

  const cancelVariables = useCallback(() => {
    setView("list");
    resetVariables();
    focusSearch();
  }, [resetVariables, focusSearch]);

  const copyVariablesAction = useCallback(
    async (afterCopy: () => void | Promise<void>) => {
      if (!variablePrompt) return;
      try {
        await copyPromptToClipboard({
          text: preview,
          promptName: variablePrompt.name,
          messageKind: variables.length === 0 ? "noVariables" : "variables",
          promptId: variablePrompt.id,
          values: buildCopyValues(),
        });
        await afterCopy();
      } catch (err) {
        reportStatusError(
          err instanceof Error && err.message
            ? err.message
            : "Could not copy prompt.",
        );
      }
    },
    [variablePrompt, preview, variables.length, buildCopyValues, reportStatusError],
  );

  const copyAndBackToList = useCallback(async () => {
    await copyVariablesAction(() => {
      setView("list");
      resetVariables();
      focusSearch();
    });
  }, [copyVariablesAction, resetVariables, focusSearch]);

  const copyVariables = useCallback(async () => {
    await copyVariablesAction(() => {});
  }, [copyVariablesAction]);

  const copyAndCloseVariables = useCallback(async () => {
    await copyVariablesAction(async () => {
      setView("list");
      resetVariables();
      await api.hideWindow();
    });
  }, [copyVariablesAction, resetVariables]);

  useEffect(() => {
    if (
      view !== "editor" &&
      view !== "variables" &&
      view !== "history" &&
      view !== "historyDetail"
    ) {
      return;
    }
    const onKeyDown = (e: KeyboardEvent) => {
      if (e.key !== "Escape" || e.ctrlKey) return;
      e.preventDefault();
      e.stopPropagation();
      if (view === "editor") closeEditor();
      else if (view === "variables") void copyAndBackToList();
      else if (view === "history") closeHistory();
      else closeHistoryDetail();
    };
    document.addEventListener("keydown", onKeyDown, true);
    return () => document.removeEventListener("keydown", onKeyDown, true);
  }, [
    view,
    closeEditor,
    copyAndBackToList,
    closeHistory,
    closeHistoryDetail,
  ]);

  const copyHistoryDetail = useCallback(async () => {
    if (!historyDetail) return;
    const edited = historyDetailContent !== historyDetail.content;
    try {
      await copyPromptToClipboard({
        text: historyDetailContent,
        promptName: historyDetail.promptName,
        messageKind:
          historyDetail.variables.length === 0 ? "noVariables" : "variables",
        promptId: historyDetail.promptId,
        values: historyDetail.variables,
        skipHistory: true,
      });
      if (edited) {
        await updateHistoryContent(historyDetail.id, historyDetailContent);
        setHistoryDetail({ ...historyDetail, content: historyDetailContent });
      }
    } catch (err) {
      reportStatusError(
        err instanceof Error && err.message
          ? err.message
          : "Could not copy from history.",
      );
    }
  }, [historyDetail, historyDetailContent, updateHistoryContent, reportStatusError]);

  const handlePruneKeep = useCallback(
    async (keep: number) => {
      setPruneMenuOpen(false);
      try {
        await pruneHistoryKeep(keep);
        setHistorySelectedIndex(0);
      } catch {
        // silent
      }
    },
    [pruneHistoryKeep],
  );

  if (view === "editor") {
    return (
      <EditorView
        editingPrompt={editingPrompt}
        editorFormRef={editorFormRef}
        editorError={editorError}
        content={editorContent}
        onContentChange={setEditorContent}
        onClose={closeEditor}
        onSave={() => void handleSave()}
      />
    );
  }

  if (view === "delete" && deletingPrompt) {
    return (
      <DeleteView
        deletingPrompt={deletingPrompt}
        onClose={closeDelete}
        onConfirm={() => void confirmDelete()}
      />
    );
  }

  if (view === "variables" && variablePrompt) {
    return (
      <VariablesView
        variablePrompt={variablePrompt}
        variables={variables}
        preview={preview}
        setPreview={setPreview}
        onVariableInput={handleVariableInput}
        onCancel={cancelVariables}
        onCopy={() => void copyVariables()}
        onCopyAndClose={() => void copyAndCloseVariables()}
      />
    );
  }

  if (view === "historyDetail" && historyDetail) {
    return (
      <HistoryDetailView
        historyDetail={historyDetail}
        historyDetailContent={historyDetailContent}
        setHistoryDetailContent={setHistoryDetailContent}
        onClose={closeHistoryDetail}
        onCopy={() => void copyHistoryDetail()}
      />
    );
  }

  if (view === "history") {
    return (
      <HistoryView
        historyQuery={historyQuery}
        setHistoryQuery={setHistoryQuery}
        setHistorySelectedIndex={setHistorySelectedIndex}
        historySearchRef={historySearchRef}
        historyListRef={historyListRef}
        pruneMenuRef={pruneMenuRef}
        filteredHistory={filteredHistory}
        historyTotalCount={historyTotalCount}
        historySelectedIndex={historySelectedIndex}
        pruneMenuOpen={pruneMenuOpen}
        setPruneMenuOpen={setPruneMenuOpen}
        focusHistorySearch={focusHistorySearch}
        onKeyDown={handleHistoryKey}
        onOpenDetail={(entry) => void openHistoryDetail(entry)}
        onDeleteItem={(id, e) => void deleteHistoryItem(id)}
        onPruneKeep={(keep) => void handlePruneKeep(keep)}
      />
    );
  }

  return (
    <ListView
      query={query}
      setQuery={setQuery}
      setSelectedIndex={setSelectedIndex}
      searchRef={searchRef}
      listRef={listRef}
      filtered={filtered}
      prompts={prompts}
      selectedIndex={selectedIndex}
      focusSearch={focusSearch}
      onKeyDown={handleListKey}
      onOpenHistory={() => void openHistory()}
      onOpenNew={openNew}
      onSelectPrompt={(p) => void selectPrompt(p)}
      onEditPrompt={openEdit}
      onDeletePrompt={openDelete}
      statusError={statusError}
    />
  );
}
