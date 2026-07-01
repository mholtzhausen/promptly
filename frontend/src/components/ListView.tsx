import type { RefObject } from "react";
import { CategoryChip } from "./CategoryChip";
import {
  DEFAULT_CATEGORY,
  allCategoriesSelected,
  categoryChipClass,
  categoryLabel,
  initialSelectedCategories,
} from "../lib/categories";
import type { FilteredPrompts } from "../lib/fuzzy";
import { flattenFilteredPrompts } from "../lib/fuzzy";
import type { CategoryDef, Prompt } from "../types";

type ListViewProps = {
  query: string;
  setQuery: (q: string) => void;
  setSelectedIndex: (i: number) => void;
  searchRef: RefObject<HTMLInputElement | null>;
  listRef: RefObject<HTMLDivElement | null>;
  categoryMenuRef: RefObject<HTMLDivElement | null>;
  filtered: FilteredPrompts;
  prompts: Prompt[];
  categories: CategoryDef[];
  selectedIndex: number;
  selectedCategories: Set<string>;
  setSelectedCategories: (next: Set<string>) => void;
  categoryMenuOpen: boolean;
  setCategoryMenuOpen: (open: boolean) => void;
  focusSearch: () => void;
  onKeyDown: (e: React.KeyboardEvent) => void;
  onOpenSettings: () => void;
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
  filtered: FilteredPrompts,
  query: string,
  selectedCategories: Set<string>,
  categories: CategoryDef[],
): string {
  const filteredFlat = flattenFilteredPrompts(filtered);
  if (prompts.length === 0) {
    return "No prompts yet. Click + to add one.";
  }

  const filteringCategories = !allCategoriesSelected(
    selectedCategories,
    categories,
  );
  const activeLabels = filteringCategories
    ? categories
        .filter((c) => selectedCategories.has(c.slug))
        .map((c) => c.label)
    : [];

  if (query && filteredFlat.length === 0) {
    if (filteringCategories && activeLabels.length > 0) {
      return `No matches for "${query}" in ${activeLabels.join(", ")}`;
    }
    return `No matches for "${query}"`;
  }

  const countLabel = `${filteredFlat.length} prompt${filteredFlat.length !== 1 ? "s" : ""}`;
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
  categories,
  selectedIndex,
  selectedCategories,
  setSelectedCategories,
  categoryMenuOpen,
  setCategoryMenuOpen,
  focusSearch,
  onKeyDown,
  onOpenSettings,
  onOpenHistory,
  onOpenNew,
  onSelectPrompt,
  onEditPrompt,
  onDeletePrompt,
  statusError,
}: ListViewProps) {
  const filteringCategories = !allCategoriesSelected(
    selectedCategories,
    categories,
  );

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
    setSelectedCategories(initialSelectedCategories(categories));
    setSelectedIndex(0);
  };

  const clearAllCategories = () => {
    setSelectedCategories(new Set());
    setSelectedIndex(0);
  };

  const showFilterSections = query.trim().length > 0;
  let rowIndex = 0;

  const renderPromptRow = (p: Prompt) => {
    const i = rowIndex;
    rowIndex += 1;
    return (
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
          {p.category !== DEFAULT_CATEGORY && (
            <CategoryChip
              label={categoryLabel(p.category, categories)}
              chipClass={categoryChipClass(p.category, categories)}
            />
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
    );
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
              next?.closest("#settings-button") ||
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
                  {categories.map((category) => {
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
                          {category.slug !== DEFAULT_CATEGORY ? (
                            <CategoryChip
                              label={category.label}
                              chipClass={category.chipClass}
                            />
                          ) : (
                            category.label
                          )}
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
          id="settings-button"
          type="button"
          title="Settings"
          aria-label="Open settings"
          onClick={onOpenSettings}
        >
          ⚙
        </button>
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
        {showFilterSections ? (
          <>
            {filtered.nameMatches.length > 0 && (
              <>
                <div className="prompt-list-heading" aria-hidden="true">
                  Name matches
                </div>
                {filtered.nameMatches.map(renderPromptRow)}
              </>
            )}
            {filtered.descMatches.length > 0 && (
              <>
                <div className="prompt-list-heading" aria-hidden="true">
                  Description matches
                </div>
                {filtered.descMatches.map(renderPromptRow)}
              </>
            )}
          </>
        ) : (
          filtered.nameMatches.map(renderPromptRow)
        )}
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
          listStatusText(
            prompts,
            filtered,
            query,
            selectedCategories,
            categories,
          )
        )}
      </div>
    </div>
  );
}
