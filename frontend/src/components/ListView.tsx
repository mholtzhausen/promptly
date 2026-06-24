import type { RefObject } from "react";
import type { Prompt } from "../types";

type ListViewProps = {
  query: string;
  setQuery: (q: string) => void;
  setSelectedIndex: (i: number) => void;
  searchRef: RefObject<HTMLInputElement | null>;
  listRef: RefObject<HTMLDivElement | null>;
  filtered: Prompt[];
  prompts: Prompt[];
  selectedIndex: number;
  focusSearch: () => void;
  onKeyDown: (e: React.KeyboardEvent) => void;
  onOpenHistory: () => void;
  onOpenNew: () => void;
  onSelectPrompt: (p: Prompt) => void;
  onEditPrompt: (p: Prompt) => void;
  onDeletePrompt: (p: Prompt) => void;
  statusError: string | null;
};

export function ListView({
  query,
  setQuery,
  setSelectedIndex,
  searchRef,
  listRef,
  filtered,
  prompts,
  selectedIndex,
  focusSearch,
  onKeyDown,
  onOpenHistory,
  onOpenNew,
  onSelectPrompt,
  onEditPrompt,
  onDeletePrompt,
  statusError,
}: ListViewProps) {
  return (
    <div className="app list-view" onKeyDown={onKeyDown}>
      <div id="top-bar" className="panel-header">
        <input
          id="search-entry"
          ref={searchRef}
          type="search"
          placeholder="Filter prompts..."
          aria-label="Filter prompts"
          value={query}
          onChange={(e) => {
            setQuery(e.target.value);
            setSelectedIndex(0);
          }}
          onBlur={(e) => {
            const next = e.relatedTarget as HTMLElement | null;
            if (
              next?.closest("#add-button") ||
              next?.closest("#history-button") ||
              next?.closest(".action-btn")
            ) {
              return;
            }
            focusSearch();
          }}
        />
        <button
          id="history-button"
          title="Copy history"
          aria-label="Open copy history"
          onClick={onOpenHistory}
        >
          ⟳
        </button>
        <button
          id="add-button"
          title="Add prompt"
          aria-label="Add new prompt"
          onClick={onOpenNew}
        >
          +
        </button>
      </div>
      <div id="prompt-list" ref={listRef} role="listbox" aria-label="Prompt templates">
        {filtered.map((p, i) => (
          <div
            key={p.id}
            role="option"
            aria-selected={i === selectedIndex}
            className={"prompt-row" + (i === selectedIndex ? " selected" : "")}
            onClick={(e) => {
              setSelectedIndex(i);
              focusSearch();
              if (e.ctrlKey || e.metaKey) {
                onEditPrompt(p);
              } else {
                onSelectPrompt(p);
              }
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
                  onEditPrompt(p);
                }}
              >
                ✎
              </button>
              <button
                className="action-btn"
                title="Delete prompt"
                onClick={(e) => {
                  e.stopPropagation();
                  onDeletePrompt(p);
                }}
              >
                ✕
              </button>
            </div>
          </div>
        ))}
      </div>
      <div
        id="status-label"
        className="panel-footer"
        aria-live="polite"
        aria-atomic="true"
      >
        {statusError ? (
          <p className="form-error">{statusError}</p>
        ) : prompts.length === 0
          ? "No prompts yet. Click + to add one."
          : query && filtered.length === 0
            ? `No matches for "${query}"`
            : `${filtered.length} prompt${filtered.length !== 1 ? "s" : ""} available`}
      </div>
    </div>
  );
}
