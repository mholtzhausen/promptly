import { useState, useEffect, useCallback, useRef } from "react";
import { request } from "./ipc";
import type {
  Prompt,
  VariableDto,
  SavePromptResult,
  SavePromptPayload,
  DeletePromptPayload,
  InterpolatePayload,
  CopyPromptPayload,
} from "./types";
import "./styles.css";

/** Fuzzy subsequence match — ported from src/popup.rs:389-401. */
function fuzzyMatch(text: string, pattern: string): boolean {
  const tl = text.toLowerCase();
  const pl = pattern.toLowerCase();
  let ti = 0;
  for (let pi = 0; pi < pl.length; pi++) {
    const idx = tl.indexOf(pl[pi], ti);
    if (idx === -1) return false;
    ti = idx + 1;
  }
  return true;
}

/** Filter prompts by fuzzy match, ordered: name → description → content. */
function filterPrompts(prompts: Prompt[], query: string): Prompt[] {
  const q = query.trim();
  if (!q) return prompts;

  const nameMatches: Prompt[] = [];
  const descMatches: Prompt[] = [];
  const contentMatches: Prompt[] = [];

  for (const p of prompts) {
    if (fuzzyMatch(p.name, q)) {
      nameMatches.push(p);
    } else if (fuzzyMatch(p.description, q)) {
      descMatches.push(p);
    } else if (fuzzyMatch(p.content, q)) {
      contentMatches.push(p);
    }
  }

  return [...nameMatches, ...descMatches, ...contentMatches];
}

type View = "list" | "editor" | "delete" | "variables";

export default function App() {
  const [view, setView] = useState<View>("list");
  const [prompts, setPrompts] = useState<Prompt[]>([]);
  const [query, setQuery] = useState("");
  const [selectedIndex, setSelectedIndex] = useState(0);

  // Editor state
  const [editingPrompt, setEditingPrompt] = useState<Prompt | null>(null);

  // Delete confirmation state
  const [deletingPrompt, setDeletingPrompt] = useState<Prompt | null>(null);

  // Variables state
  const [variablePrompt, setVariablePrompt] = useState<Prompt | null>(null);
  const [variables, setVariables] = useState<VariableDto[]>([]);
  const [variableValues, setVariableValues] = useState<Record<string, string>>({});
  const [preview, setPreview] = useState("");

  const searchRef = useRef<HTMLInputElement>(null);
  const listRef = useRef<HTMLDivElement>(null);
  const editorFormRef = useRef<HTMLFormElement>(null);
  const [editorError, setEditorError] = useState<string | null>(null);

  const focusSearch = useCallback(() => {
    const attempt = () => searchRef.current?.focus({ preventScroll: true });
    attempt();
    requestAnimationFrame(attempt);
    window.setTimeout(attempt, 0);
    window.setTimeout(attempt, 50);
  }, []);

  window.__promptlyFocusSearch = focusSearch;

  // ── Load prompts (called on mount and on show) ──────────────────────
  const loadPrompts = useCallback(async () => {
    try {
      const list = await request<Prompt[]>("listPrompts");
      setPrompts(list);
    } catch {
      // silently ignore
    }
  }, []);

  // Called by Rust when the window becomes visible.
  window.__promptlyOnShow = useCallback(() => {
    setView("list");
    setQuery("");
    setSelectedIndex(0);
    loadPrompts();
    focusSearch();
  }, [loadPrompts, focusSearch]);

  // Initial load
  useEffect(() => {
    loadPrompts();
  }, [loadPrompts]);

  // Keep the filter input focused whenever the list view is active.
  useEffect(() => {
    if (view !== "list") return;
    focusSearch();
    window.addEventListener("focus", focusSearch);
    return () => window.removeEventListener("focus", focusSearch);
  }, [view, focusSearch]);

  // Route printable keys to the filter when focus is elsewhere in the list view.
  useEffect(() => {
    if (view !== "list") return;
    const onKeyDown = (e: KeyboardEvent) => {
      if (e.target === searchRef.current) return;
      if (e.ctrlKey || e.metaKey || e.altKey) return;
      if (e.key.length !== 1) return;
      e.preventDefault();
      searchRef.current?.focus();
      setQuery((prev) => prev + e.key);
      setSelectedIndex(0);
    };
    document.addEventListener("keydown", onKeyDown, true);
    return () => document.removeEventListener("keydown", onKeyDown, true);
  }, [view]);

  // ── Filtered list ──────────────────────────────────────────────────
  const filtered = filterPrompts(prompts, query);

  // Ensure selectedIndex stays in bounds when the filter changes.
  useEffect(() => {
    if (filtered.length === 0) {
      setSelectedIndex(0);
    } else if (selectedIndex >= filtered.length) {
      setSelectedIndex(filtered.length - 1);
    }
  }, [filtered.length, selectedIndex]);

  // Scroll the highlighted row into view.
  useEffect(() => {
    if (view !== "list" || !listRef.current) return;
    const row = listRef.current.children[selectedIndex] as
      | HTMLElement
      | undefined;
    row?.scrollIntoView({ block: "nearest" });
  }, [selectedIndex, filtered, view]);

  // ── Prompt selection (copy or show variables) ──────────────────────
  const selectPrompt = useCallback(async (prompt: Prompt) => {
    try {
      const vars = await request<VariableDto[]>("variablesForTemplate", {
        content: prompt.content,
      } as never);
      if (vars.length === 0) {
        // Copy immediately.
        await copyPrompt(prompt.content, prompt.name, "noVariables");
        request("hideWindow");
      } else {
        setVariablePrompt(prompt);
        setVariables(vars);
        const values: Record<string, string> = {};
        for (const v of vars) {
          values[v.name] = v.defaultValue;
        }
        setVariableValues(values);
        // Compute initial preview.
        const interp = await request<string>("interpolate", {
          template: prompt.content,
          values: vars.map((v) => ({ name: v.name, value: values[v.name] })),
        } as InterpolatePayload);
        setPreview(interp);
        setView("variables");
      }
    } catch {
      // silently ignore
    }
  }, []);

  // ── List keyboard navigation ──────────────────────────────────────
  const handleListKey = useCallback(
    (e: React.KeyboardEvent) => {
      if (view !== "list") return;

      switch (e.key) {
        case "Escape":
          e.preventDefault();
          request("hideWindow");
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
          if (max > 0)
            setSelectedIndex((prev) => (prev + max - 1) % max);
          break;
        }
        case "Enter": {
          e.preventDefault();
          const prompt = filtered[selectedIndex];
          if (!prompt) return;
          if (e.ctrlKey || e.metaKey) {
            openEdit(prompt);
          } else {
            selectPrompt(prompt);
          }
          break;
        }
      }
    },
    [view, filtered, selectedIndex, selectPrompt],
  );

  async function copyPrompt(
    text: string,
    promptName: string,
    messageKind: "noVariables" | "variables",
  ) {
    await request("copyPrompt", {
      text,
      promptName,
      messageKind,
    } as CopyPromptPayload);
  }

  // ── Editor ─────────────────────────────────────────────────────────
  function openNew() {
    setEditingPrompt(null);
    setEditorError(null);
    setView("editor");
  }

  function openEdit(p: Prompt) {
    setEditingPrompt(p);
    setEditorError(null);
    setView("editor");
  }

  const closeEditor = useCallback(() => {
    setView("list");
    setEditingPrompt(null);
    setEditorError(null);
    focusSearch();
  }, [focusSearch]);

  async function handleSave() {
    const form = editorFormRef.current;
    if (!form) return;

    setEditorError(null);
    const data = new FormData(form);
    const id = editingPrompt?.id ?? null;
    const name = ((data.get("name") as string) ?? "").trim();
    const description = ((data.get("description") as string) ?? "").trim();
    const content = (data.get("content") as string) ?? "";

    if (!name || !description || !content.trim()) {
      setEditorError("Name, description, and content are all required.");
      return;
    }

    const payload: SavePromptPayload = { id, name, description, content };
    try {
      const result = await request<SavePromptResult>("savePrompt", payload);
      if (!result?.saved) {
        setEditorError("Could not save — check that all fields are filled in.");
        return;
      }
    } catch (err) {
      const msg =
        err instanceof Error && err.message
          ? err.message
          : "Could not save the prompt. Please try again.";
      setEditorError(msg);
      return;
    }

    try {
      await loadPrompts();
    } catch {
      // Save succeeded; reload failure is non-fatal.
    }
    setView("list");
    setEditingPrompt(null);
    setEditorError(null);
  }

  // ── Delete confirmation ────────────────────────────────────────────
  function openDelete(p: Prompt) {
    setDeletingPrompt(p);
    setView("delete");
  }

  function closeDelete() {
    setView("list");
    setDeletingPrompt(null);
  }

  async function confirmDelete() {
    if (!deletingPrompt) return;
    const payload: DeletePromptPayload = {
      id: deletingPrompt.id,
      name: deletingPrompt.name,
    };
    try {
      await request("deletePrompt", payload);
      await loadPrompts();
    } catch {
      // silent
    }
    setView("list");
    setDeletingPrompt(null);
  }

  // ── Variables ─────────────────────────────────────────────────────
  async function onVariableChange() {
    if (!variablePrompt) return;
    try {
      const interp = await request<string>("interpolate", {
        template: variablePrompt.content,
        values: Object.entries(variableValues).map(([name, value]) => ({
          name,
          value,
        })),
      } as InterpolatePayload);
      setPreview(interp);
    } catch {
      // silent
    }
  }

  const handleVariableInput = useCallback(
    (name: string, value: string) => {
      setVariableValues((prev) => ({ ...prev, [name]: value }));
    },
    [],
  );

  // Recompute preview when variableValues change.
  useEffect(() => {
    onVariableChange();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [variableValues]);

  const cancelVariables = useCallback(() => {
    setView("list");
    setVariablePrompt(null);
    setVariables([]);
    setVariableValues({});
    setPreview("");
    focusSearch();
  }, [focusSearch]);

  // Escape from editor or fill panes returns to the filter list (not hide window).
  useEffect(() => {
    if (view !== "editor" && view !== "variables") return;
    const onKeyDown = (e: KeyboardEvent) => {
      if (e.key !== "Escape") return;
      e.preventDefault();
      if (view === "editor") closeEditor();
      else cancelVariables();
    };
    document.addEventListener("keydown", onKeyDown, true);
    return () => document.removeEventListener("keydown", onKeyDown, true);
  }, [view, closeEditor, cancelVariables]);

  async function copyAndCloseVariables() {
    if (!variablePrompt) return;
    try {
      await copyPrompt(preview, variablePrompt.name, "variables");
      setView("list");
      setVariablePrompt(null);
    } catch {
      // silent
    }
  }

  // ── Render ─────────────────────────────────────────────────────────

  if (view === "editor") {
    const p = editingPrompt;
    return (
      <div className="app editor-view">
        <h1 className="editor-header panel-header">
          {p ? "Edit Prompt Template" : "New Prompt Template"}
        </h1>
        <div className="editor-body">
          <form ref={editorFormRef} noValidate>
            <label>
              Prompt Name
              <input
                name="name"
                type="text"
                defaultValue={p?.name ?? ""}
                placeholder="e.g. git-commit"
              />
            </label>
            <label>
              Description
              <input
                name="description"
                type="text"
                defaultValue={p?.description ?? ""}
                placeholder="Short summary shown next to the title"
              />
            </label>
            <label className="template-content-field">
              Template Content
              <textarea
                name="content"
                className="mono"
                defaultValue={p?.content ?? ""}
              />
            </label>
          </form>
        </div>
        <div className="editor-footer panel-footer">
          <p className="help">
            Use {"{{name|type|default|desc}}"} placeholders. Types: text,
            number, option, multiline.
          </p>
          {editorError && <p className="form-error">{editorError}</p>}
          <div className="buttons">
            <button type="button" onClick={closeEditor}>
              Cancel
            </button>
            <button type="button" className="primary" onClick={handleSave}>
              {p ? "Update" : "Save"}
            </button>
          </div>
        </div>
      </div>
    );
  }

  if (view === "delete" && deletingPrompt) {
    return (
      <div
        className="app"
        onKeyDown={(e) => e.key === "Escape" && closeDelete()}
      >
        <h1>Delete Prompt Template</h1>
        <p className="confirm-msg">
          Delete &lsquo;{deletingPrompt.name}&rsquo;? This cannot be undone.
        </p>
        <div className="buttons">
          <button type="button" onClick={closeDelete}>
            Cancel
          </button>
          <button type="button" className="danger" onClick={confirmDelete}>
            Delete
          </button>
        </div>
      </div>
    );
  }

  if (view === "variables" && variablePrompt) {
    return (
      <div className="app variables-view">
        <h1 className="variables-header panel-header">
          Fill in variables for &lsquo;{variablePrompt.name}&rsquo;
        </h1>
        <div className="variables-body">
          {variables.map((v) => (
            <label key={v.name} className="variable-field">
              <span className="var-name">{v.name}</span>
              {v.description && (
                <span className="var-desc">{v.description}</span>
              )}
              {v.kind === "text" && (
                <input
                  type="text"
                  defaultValue={v.defaultValue}
                  onChange={(e) => handleVariableInput(v.name, e.target.value)}
                />
              )}
              {v.kind === "number" && (
                <input
                  type="number"
                  defaultValue={
                    v.defaultValue ? parseFloat(v.defaultValue) || 0 : 0
                  }
                  onChange={(e) => handleVariableInput(v.name, e.target.value)}
                />
              )}
              {v.kind === "option" && (
                <select
                  defaultValue={v.options[0] ?? ""}
                  onChange={(e) => handleVariableInput(v.name, e.target.value)}
                >
                  {v.options.map((opt) => (
                    <option key={opt} value={opt}>
                      {opt}
                    </option>
                  ))}
                </select>
              )}
              {v.kind === "multiline" && (
                <textarea
                  className="mono multiline"
                  defaultValue={v.defaultValue}
                  onChange={(e) => handleVariableInput(v.name, e.target.value)}
                />
              )}
            </label>
          ))}
          <label className="preview-field">
            Prompt to copy
            <textarea
              className="mono multiline preview"
              value={preview}
              onChange={(e) => setPreview(e.target.value)}
            />
          </label>
        </div>
        <div className="variables-footer panel-footer">
          <div className="buttons">
            <button type="button" onClick={cancelVariables}>
              Cancel
            </button>
            <button
              type="button"
              className="primary"
              onClick={copyAndCloseVariables}
            >
              Copy &amp; Close
            </button>
          </div>
        </div>
      </div>
    );
  }

  // Default: list view
  return (
    <div className="app list-view" onKeyDown={handleListKey}>
      <div id="top-bar" className="panel-header">
        <input
          id="search-entry"
          ref={searchRef}
          type="search"
          placeholder="Filter prompts..."
          value={query}
          onChange={(e) => {
            setQuery(e.target.value);
            setSelectedIndex(0);
          }}
          onInput={(e) => {
            setQuery(e.currentTarget.value);
            setSelectedIndex(0);
          }}
          onBlur={(e) => {
            const next = e.relatedTarget as HTMLElement | null;
            if (next?.closest("#add-button") || next?.closest(".action-btn")) {
              return;
            }
            focusSearch();
          }}
        />
        <button id="add-button" title="Add prompt" onClick={openNew}>
          +
        </button>
      </div>
      <div id="prompt-list" ref={listRef}>
        {filtered.map((p, i) => (
          <div
            key={p.id}
            className={
              "prompt-row" + (i === selectedIndex ? " selected" : "")
            }
            onClick={() => {
              setSelectedIndex(i);
              focusSearch();
              selectPrompt(p);
            }}
          >
            <div className="prompt-text">
              <span className="prompt-title">{p.name}</span>
              <span className="prompt-description">{p.description}</span>
            </div>
            <div className="prompt-actions">
              <button
                className="action-btn"
                title="Edit prompt"
                onClick={(e) => {
                  e.stopPropagation();
                  openEdit(p);
                }}
              >
                ✎
              </button>
              <button
                className="action-btn"
                title="Delete prompt"
                onClick={(e) => {
                  e.stopPropagation();
                  openDelete(p);
                }}
              >
                ✕
              </button>
            </div>
          </div>
        ))}
      </div>
      <div id="status-label" className="panel-footer">
        {prompts.length === 0
          ? "No prompts yet. Click + to add one."
          : query && filtered.length === 0
            ? `No matches for "${query}"`
            : `${filtered.length} prompt${filtered.length !== 1 ? "s" : ""} available`}
      </div>
    </div>
  );
}
