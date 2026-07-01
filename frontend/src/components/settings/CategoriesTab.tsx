import { useEffect, useRef, useState } from "react";
import type { CategoryDef } from "../../types";
import { DEFAULT_CATEGORY } from "../../lib/categories";
import {
  defaultChipClassForNewCategory,
  resolveCategoryChip,
  uniqueSlugFromLabel,
} from "../../lib/categoryColors";
import { CategoryEditDialog } from "./CategoryEditDialog";

type CategoryRow = CategoryDef & { previousSlug?: string };

type CategoriesTabProps = {
  categories: CategoryDef[];
  saving: boolean;
  onSave: (categories: CategoryDef[]) => void;
  onEditorOpenChange?: (open: boolean) => void;
};

type DialogState =
  | { mode: "add"; anchor: DOMRect; pendingChipClass: string }
  | { mode: "edit"; anchor: DOMRect; index: number };

function toRows(categories: CategoryDef[]): CategoryRow[] {
  return categories.map((c) => ({ ...c }));
}

function toPayload(rows: CategoryRow[]): CategoryDef[] {
  return rows.map((row) => ({
    slug: row.slug,
    label: row.label,
    chipClass: row.chipClass,
    previousSlug: row.previousSlug,
  }));
}

export function CategoriesTab({
  categories,
  saving,
  onSave,
  onEditorOpenChange,
}: CategoriesTabProps) {
  const [rows, setRows] = useState<CategoryRow[]>(() => toRows(categories));
  const [dialog, setDialog] = useState<DialogState | null>(null);
  const addButtonRef = useRef<HTMLButtonElement>(null);

  useEffect(() => {
    setRows(toRows(categories));
  }, [categories]);

  useEffect(() => {
    onEditorOpenChange?.(dialog !== null);
  }, [dialog, onEditorOpenChange]);

  const persist = (next: CategoryRow[]) => {
    setRows(next);
    onSave(toPayload(next));
  };

  const closeDialog = () => setDialog(null);

  const openAddDialog = () => {
    const el = addButtonRef.current;
    if (!el) return;
    setDialog({
      mode: "add",
      anchor: el.getBoundingClientRect(),
      pendingChipClass: defaultChipClassForNewCategory(
        rows.map((r) => r.chipClass),
      ),
    });
  };

  const openEditDialog = (index: number, anchor: DOMRect) => {
    setDialog({ mode: "edit", anchor, index });
  };

  const applyEdit = (index: number, label: string, chipClass: string) => {
    const trimmed = label.trim();
    if (!trimmed) return;
    const row = rows[index];
    const slug = uniqueSlugFromLabel(
      trimmed,
      rows.map((r) => r.slug),
      row.slug,
    );
    const next = rows.map((item, i) => {
      if (i !== index) return item;
      return {
        ...item,
        label: trimmed,
        chipClass,
        slug: row.slug === DEFAULT_CATEGORY ? DEFAULT_CATEGORY : slug,
        previousSlug:
          row.slug === DEFAULT_CATEGORY || slug === row.slug
            ? item.previousSlug
            : (item.previousSlug ?? row.slug),
      };
    });
    persist(next);
  };

  const applyAdd = (label: string, chipClass: string) => {
    const trimmed = label.trim();
    if (!trimmed) return -1;
    const slug = uniqueSlugFromLabel(trimmed, rows.map((r) => r.slug));
    const next = [
      ...rows,
      {
        slug,
        label: trimmed,
        chipClass,
      },
    ];
    persist(next);
    return next.length - 1;
  };

  const handleChange = (label: string, chipClass: string) => {
    if (!dialog) return;

    if (dialog.mode === "edit") {
      applyEdit(dialog.index, label, chipClass);
      return;
    }

    const trimmed = label.trim();
    if (!trimmed) {
      setDialog({ ...dialog, pendingChipClass: chipClass });
      return;
    }

    const newIndex = applyAdd(label, chipClass);
    if (newIndex >= 0) {
      setDialog({ mode: "edit", anchor: dialog.anchor, index: newIndex });
    }
  };

  const handleDelete = (index: number) => {
    const row = rows[index];
    if (row.slug === DEFAULT_CATEGORY) return;
    persist(rows.filter((_, i) => i !== index));
    closeDialog();
  };

  const dialogRow = dialog?.mode === "edit" ? rows[dialog.index] : null;

  return (
    <section aria-label="Category settings" className="categories-tab">
      <h2>Categories</h2>
      <p className="settings-hint">
        Manage categories available when creating and filtering prompts.
      </p>
      <div className="category-chip-flow" role="list" aria-label="Categories">
        {rows.map((row, index) => {
          const { className, style } = resolveCategoryChip(row.chipClass);
          return (
            <button
              key={row.slug}
              type="button"
              role="listitem"
              className={`${className} category-chip-button`}
              style={style}
              disabled={saving}
              onClick={(e) => {
                openEditDialog(index, e.currentTarget.getBoundingClientRect());
              }}
            >
              {row.label}
            </button>
          );
        })}
        <button
          ref={addButtonRef}
          type="button"
          className="category-add-button"
          aria-label="Add category"
          disabled={saving}
          onClick={openAddDialog}
        >
          +
        </button>
      </div>
      {saving && <p className="settings-status">Saving…</p>}
      {dialog?.mode === "add" && (
        <CategoryEditDialog
          anchorRect={dialog.anchor}
          initialLabel=""
          initialChipClass={dialog.pendingChipClass}
          onChange={handleChange}
          onClose={closeDialog}
        />
      )}
      {dialog?.mode === "edit" && dialogRow && (
        <CategoryEditDialog
          anchorRect={dialog.anchor}
          initialLabel={dialogRow.label}
          initialChipClass={dialogRow.chipClass}
          canDelete={dialogRow.slug !== DEFAULT_CATEGORY}
          onDelete={() => handleDelete(dialog.index)}
          onChange={handleChange}
          onClose={closeDialog}
        />
      )}
    </section>
  );
}
