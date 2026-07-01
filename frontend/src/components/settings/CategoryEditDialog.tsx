import { useEffect, useLayoutEffect, useRef, useState } from "react";
import { chipClassForStored } from "../../lib/categoryColors";
import { computePopoverPosition } from "../../lib/popoverPlacement";
import { PastelColorPicker } from "./PastelColorPicker";

type CategoryEditDialogProps = {
  anchorRect: DOMRect;
  initialLabel: string;
  initialChipClass: string;
  onChange: (label: string, chipClass: string) => void;
  onClose: () => void;
  canDelete?: boolean;
  onDelete?: () => void;
};

export function CategoryEditDialog({
  anchorRect,
  initialLabel,
  initialChipClass,
  onChange,
  onClose,
  canDelete = false,
  onDelete,
}: CategoryEditDialogProps) {
  const [label, setLabel] = useState(initialLabel);
  const [chipClass, setChipClass] = useState(() =>
    chipClassForStored(initialChipClass),
  );
  const [position, setPosition] = useState<{ top: number; left: number } | null>(
    null,
  );
  const panelRef = useRef<HTMLDivElement>(null);
  const labelRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    setLabel(initialLabel);
    setChipClass(chipClassForStored(initialChipClass));
  }, [initialLabel, initialChipClass]);

  useLayoutEffect(() => {
    const el = panelRef.current;
    if (!el) return;
    const rect = el.getBoundingClientRect();
    setPosition(computePopoverPosition(anchorRect, rect.width, rect.height));
  }, [anchorRect, canDelete]);

  useEffect(() => {
    const t = window.setTimeout(() => {
      labelRef.current?.focus();
      labelRef.current?.select();
    }, 0);
    return () => window.clearTimeout(t);
  }, []);

  useEffect(() => {
    const onMouseDown = (e: MouseEvent) => {
      if (panelRef.current && !panelRef.current.contains(e.target as Node)) {
        onClose();
      }
    };
    const onKeyDown = (e: KeyboardEvent) => {
      if (e.key !== "Escape" || e.ctrlKey) return;
      e.preventDefault();
      e.stopPropagation();
      onClose();
    };
    document.addEventListener("mousedown", onMouseDown);
    document.addEventListener("keydown", onKeyDown, true);
    return () => {
      document.removeEventListener("mousedown", onMouseDown);
      document.removeEventListener("keydown", onKeyDown, true);
    };
  }, [onClose]);

  return (
    <div
      ref={panelRef}
      className="category-edit-dialog"
      role="dialog"
      aria-label={canDelete ? "Edit category" : "Add category"}
      style={
        position
          ? { top: position.top, left: position.left, visibility: "visible" }
          : { visibility: "hidden" }
      }
    >
      <div className="category-edit-dialog__name-row">
        <input
          ref={labelRef}
          className="category-edit-dialog__input"
          type="text"
          aria-label="Category name"
          placeholder="Category name"
          value={label}
          onChange={(e) => {
            const next = e.target.value;
            setLabel(next);
            onChange(next, chipClass);
          }}
        />
        {canDelete && onDelete ? (
          <button
            type="button"
            className="category-edit-dialog__delete"
            aria-label="Delete category"
            onClick={onDelete}
          >
            🗑
          </button>
        ) : null}
      </div>
      <PastelColorPicker
        value={chipClass}
        onChange={(next) => {
          setChipClass(next);
          onChange(label, next);
        }}
      />
    </div>
  );
}
