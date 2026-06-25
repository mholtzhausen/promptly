import type { RefObject } from "react";
import {
  CATEGORIES,
  FILTERABLE_CATEGORY_SLUGS,
  allFilterableCategoriesSelected,
  categoryChipClass,
  categoryLabel,
} from "../lib/categories";
import type { Prompt } from "../types";

type ListViewProps = {
  query: string;
  setQuery: (q: string) => void;
  setSelectedIndex: (i: number) => void;
  searchRef: RefObject<HTMLInputElement | null>;
  listRef: RefObject<HTMLDivElement | null>;
  categoryMenuRef: RefObject<HTMLDivElement | null>;
  filtered: Prompt[];
  prompts: Prompt[];
  selectedIndex: number;
  selectedCategories: Set<string>;
  setSelectedCategories: (next: Set<string>) => void;
  categoryMenuOpen: boolean;
  setCategoryMenuOpen: (open: boolean) => void;
  focusSearch: () => void;
  onKeyDown: (e: React.KeyboardEvent) => void;
  onOpenHistory: () => void;
  onOpenNew: () => void;
  onSelectPrompt: (p: Prompt) => void;
  onEditPrompt: (p: Prompt) => void;
  onDeletePrompt: (p: Prompt) => void;
  statusError: string | null;
};

function categoryCount(prompts: Prompt[], slug: string): number {
  return prompts.filter((p) => p.category === slug).length;
}

function listStatusText(
  prompts: Prompt[],
  filtered: Prompt[],
  query: string,
  selectedCategories: Set<string>,
): string {
  if (prompts.length === 0) {
    return "No prompts yet. Click + to add one.";
  }

  const filteringCategories = !allFilterableCategoriesSelected(selectedCategories);
  const activeLabels = filteringCategories
    ? CATEGORIES.filter((c) => selectedCategories.has(c.slug)).map((c) => c.label)
    : [];

  if (query && filtered.length === 0) {
    if (filteringCategories && activeLabels.length > 0) {
      return `No matches for "${query}" in ${activeLabels.join(", ")}`;
    }
    return `No matches for "${query}"`;
  }

  const countLabel = `${filtered.length} prompt${filtered.length !== 1 ? "s" : ""}`;
  if (filteringCategories && activeLabels.length > 0) {
    return `${countLabel} · ${activeLabels.join(", ")}`;
  }
  return `${countLabel} available`;
}

export function ListView({
  query,
  setQuery,
  setSelectedIndex,
  searchRef,
  listRef,
  categoryMenuRef,
  filtered,
  prompts,
  selectedIndex,
  selectedCategories,
  setSelectedCategories,
  categoryMenuOpen,
  setCategoryMenuOpen,
  focusSearch,
  onKeyDown,
  onOpenHistory,
  onOpenNew,
  onSelectPrompt,
  onEditPrompt,
  onDeletePrompt,
  statusError,
}: ListViewProps) {
  const filteringCategories = !allFilterableCategoriesSelected(selectedCategories);

  const toggleCategory = (slug: string, checked: boolean) => {
    const next = new Set(selectedCategories);
    if (checked) {
      next.add(slug);
    } else {
      next.delete(slug);
    }
    setSelectedCategories(next);
    setSelectedIndex(0);
  };

  const selectAllCategories = () => {
    setSelectedCategories(new Set(FILTERABLE_CATEGORY_SLUGS));
    setSelectedIndex(0);
  };

  const clearAllCategories = () => {
    setSelectedCategories(new Set());
    setSelectedIndex(0);
  };

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
              next?.closest("#categories-button") ||
              next?.closest(".category-filter-menu") ||
              next?.closest(".action-btn")
            ) {
              return;
            }
            focusSearch();
          }}
        />
        <div className="category-filter-wrap" ref={categoryMenuRef}>
          {categoryMenuOpen && (
            <div
              className="category-filter-menu"
              role="group"
              aria-label="Filter by category"
            >
              <div className="category-filter-actions">
                <button type="button" onClick={selectAllCategories}>
                  Select all
                </button>
                <button type="button" onClick={clearAllCategories}>
                  Clear all
                </button>
              </div>
              <table className="category-filter-table">
                <tbody>
                  {CATEGORIES.map((category) => {
                    const count = categoryCount(prompts, category.slug);
                    if (count === 0) return null;
                    const checked = selectedCategories.has(category.slug);
                    return (
                      <tr
                        key={category.slug}
                        className="category-filter-row"
                        onClick={() => toggleCategory(category.slug, !checked)}
                      >
                        <td className="category-filter-check">
                          <input
                            type="checkbox"
                            checked={checked}
                            tabIndex={-1}
                            aria-label={category.label}
                            onChange={(e) =>
                              toggleCategory(category.slug, e.target.checked)
                            }
                            onClick={(e) => e.stopPropagation()}
                          />
                        </td>
                        <td className="category-filter-label">
                          {category.label}
                        </td>
                        <td className="category-filter-count">{count}</td>
                      </tr>
                    );
                  })}
                </tbody>
              </table>
            </div>
          )}
          <button
            id="categories-button"
            type="button"
            title="Filter categories"
            aria-label="Filter by category"
            aria-expanded={categoryMenuOpen}
            aria-haspopup="true"
            className={
              filteringCategories ? "categories-button--active" : undefined
            }
            onClick={() => setCategoryMenuOpen(!categoryMenuOpen)}
          >
            ☰
          </button>
        </div>
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
              {p.category !== "general" && (
                <span
                  className={`prompt-category ${categoryChipClass(p.category)}`}
                >
                  {categoryLabel(p.category)}
                </span>
              )}
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
        ) : (
          listStatusText(prompts, filtered, query, selectedCategories)
        )}
      </div>
    </div>
  );
}
