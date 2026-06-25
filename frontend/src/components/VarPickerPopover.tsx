import {
  useEffect,
  useLayoutEffect,
  useRef,
  useState,
  type KeyboardEvent,
} from "react";
import { computePopoverPosition } from "../lib/popoverPlacement";
import { varChipLabel, type VarAttrs } from "../lib/templateVars";

export type VarPickerPopoverProps = {
  variables: VarAttrs[];
  anchorRect: DOMRect;
  onSelect: (attrs: VarAttrs) => void;
  onClose: () => void;
};

export function VarPickerPopover({
  variables,
  anchorRect,
  onSelect,
  onClose,
}: VarPickerPopoverProps) {
  const [position, setPosition] = useState<{ top: number; left: number } | null>(
    null,
  );
  const [selectedIndex, setSelectedIndex] = useState(0);
  const panelRef = useRef<HTMLDivElement>(null);
  const itemRefs = useRef<(HTMLButtonElement | null)[]>([]);

  useLayoutEffect(() => {
    const el = panelRef.current;
    if (!el) return;
    const rect = el.getBoundingClientRect();
    setPosition(computePopoverPosition(anchorRect, rect.width, rect.height));
  }, [anchorRect, variables.length]);

  useEffect(() => {
    if (variables.length === 0) return;
    itemRefs.current[selectedIndex]?.focus();
  }, [selectedIndex, variables.length]);

  useEffect(() => {
    const onMouseDown = (e: MouseEvent) => {
      if (panelRef.current && !panelRef.current.contains(e.target as Node)) {
        onClose();
      }
    };
    const onKeyDown = (e: globalThis.KeyboardEvent) => {
      if (e.key === "Escape") onClose();
    };
    document.addEventListener("mousedown", onMouseDown);
    document.addEventListener("keydown", onKeyDown, true);
    return () => {
      document.removeEventListener("mousedown", onMouseDown);
      document.removeEventListener("keydown", onKeyDown, true);
    };
  }, [onClose]);

  function onListKeyDown(e: KeyboardEvent) {
    if (variables.length === 0) return;
    if (e.key === "ArrowDown") {
      e.preventDefault();
      setSelectedIndex((i) => (i + 1) % variables.length);
    } else if (e.key === "ArrowUp") {
      e.preventDefault();
      setSelectedIndex((i) => (i - 1 + variables.length) % variables.length);
    } else if (e.key === "Enter") {
      e.preventDefault();
      const v = variables[selectedIndex];
      if (v) onSelect(v);
    }
  }

  return (
    <div
      ref={panelRef}
      className="var-picker-popover"
      style={{
        top: position?.top ?? anchorRect.top,
        left: position?.left ?? anchorRect.left,
        visibility: position ? "visible" : "hidden",
      }}
      role="dialog"
      aria-label="Insert existing variable"
      onKeyDown={onListKeyDown}
    >
      {variables.length === 0 ? (
        <p className="var-picker-empty">No variables in this template yet.</p>
      ) : (
        <ul className="var-picker-list" role="listbox">
          {variables.map((v, i) => (
            <li key={v.name} role="presentation">
              <button
                ref={(el) => {
                  itemRefs.current[i] = el;
                }}
                type="button"
                role="option"
                aria-selected={i === selectedIndex}
                className={i === selectedIndex ? "selected" : undefined}
                onClick={() => onSelect(v)}
              >
                <span className="var-picker-label">{varChipLabel(v, true)}</span>
                <span className="var-picker-meta">{v.name}</span>
                <span className="var-picker-type">{v.type}</span>
              </button>
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}
