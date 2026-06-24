import type { RefObject } from "react";
import type { HistoryListItem } from "../types";
import { HistoryTitleText } from "../lib/historyTitle";
import { PRUNE_KEEP_OPTIONS } from "../lib/view";

type HistoryViewProps = {
  historyQuery: string;
  setHistoryQuery: (q: string) => void;
  setHistorySelectedIndex: (i: number) => void;
  historySearchRef: RefObject<HTMLInputElement | null>;
  historyListRef: RefObject<HTMLDivElement | null>;
  pruneMenuRef: RefObject<HTMLDivElement | null>;
  filteredHistory: HistoryListItem[];
  historyTotalCount: number;
  historySelectedIndex: number;
  pruneMenuOpen: boolean;
  setPruneMenuOpen: (open: boolean | ((prev: boolean) => boolean)) => void;
  focusHistorySearch: () => void;
  onKeyDown: (e: React.KeyboardEvent) => void;
  onOpenDetail: (entry: HistoryListItem) => void;
  onDeleteItem: (id: number, e?: React.MouseEvent) => void;
  onPruneKeep: (keep: number) => void;
};

export function HistoryView({
  historyQuery,
  setHistoryQuery,
  setHistorySelectedIndex,
  historySearchRef,
  historyListRef,
  pruneMenuRef,
  filteredHistory,
  historyTotalCount,
  historySelectedIndex,
  pruneMenuOpen,
  setPruneMenuOpen,
  focusHistorySearch,
  onKeyDown,
  onOpenDetail,
  onDeleteItem,
  onPruneKeep,
}: HistoryViewProps) {
  const showPruneWarning = historyTotalCount >= 1000;

  return (
    <div className="app history-view" onKeyDown={onKeyDown}>
      <div id="history-top-bar" className="panel-header">
        <input
          id="history-search-entry"
          ref={historySearchRef}
          type="search"
          placeholder="Filter history..."
          value={historyQuery}
          onChange={(e) => {
            setHistoryQuery(e.target.value);
            setHistorySelectedIndex(0);
          }}
          onBlur={(e) => {
            const next = e.relatedTarget as HTMLElement | null;
            if (
              next?.closest(".action-btn") ||
              next?.closest(".history-prune-wrap")
            ) {
              return;
            }
            focusHistorySearch();
          }}
        />
      </div>
      <div id="history-list" ref={historyListRef}>
        {filteredHistory.map((entry, i) => (
          <div
            key={entry.id}
            className={
              "history-row" + (i === historySelectedIndex ? " selected" : "")
            }
            onClick={() => {
              setHistorySelectedIndex(i);
              focusHistorySearch();
              onOpenDetail(entry);
            }}
          >
            <HistoryTitleText title={entry.title} />
            <div className="prompt-actions">
              <button
                className="action-btn"
                title="Delete history entry"
                onClick={(e) => onDeleteItem(entry.id, e)}
              >
                ✕
              </button>
            </div>
          </div>
        ))}
      </div>
      <div id="history-status-label" className="panel-footer history-footer">
        {showPruneWarning && (
          <p className="history-warning">1000+ entries — consider pruning.</p>
        )}
        <div className="history-footer-row">
          <span className="history-status-text">
            {historyTotalCount === 0
              ? "No history yet. Copy a prompt to record it."
              : historyQuery && filteredHistory.length === 0
                ? `No matches for "${historyQuery}"`
                : `${filteredHistory.length} entr${filteredHistory.length !== 1 ? "ies" : "y"} shown`}
          </span>
          {historyTotalCount > 0 && (
            <div className="history-prune-wrap" ref={pruneMenuRef}>
              {pruneMenuOpen && (
                <div className="history-prune-menu" role="menu">
                  {PRUNE_KEEP_OPTIONS.map((keep) => (
                    <button
                      key={keep}
                      type="button"
                      role="menuitem"
                      onClick={() => onPruneKeep(keep)}
                    >
                      Keep last {keep}
                    </button>
                  ))}
                </div>
              )}
              <button
                id="history-prune-button"
                type="button"
                aria-expanded={pruneMenuOpen}
                aria-haspopup="menu"
                onClick={() => setPruneMenuOpen((open) => !open)}
              >
                Prune
              </button>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
