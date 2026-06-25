import { useEffect, useLayoutEffect, useRef, useState, type ReactNode } from "react";
import { computePopoverPosition } from "../lib/popoverPlacement";
import type { VarAttrs, VarType } from "../lib/templateVars";

export type VarEditPopoverProps = {
  attrs: VarAttrs;
  anchorRect: DOMRect;
  onDone: (attrs: VarAttrs) => void;
  onDelete: () => void;
  onClose: () => void;
};

const VAR_TYPES: VarType[] = ["text", "number", "option", "multiline"];

type RowProps = {
  label: string;
  children: ReactNode;
};

function Row({ label, children }: RowProps) {
  return (
    <tr>
      <th scope="row">{label}</th>
      <td>{children}</td>
    </tr>
  );
}

export function VarEditPopover({
  attrs: initial,
  anchorRect,
  onDone,
  onDelete,
  onClose,
}: VarEditPopoverProps) {
  const [attrs, setAttrs] = useState<VarAttrs>(initial);
  const [error, setError] = useState<string | null>(null);
  const [position, setPosition] = useState<{ top: number; left: number } | null>(
    null,
  );
  const [focused, setFocused] = useState(false);
  const panelRef = useRef<HTMLDivElement>(null);
  const nameRef = useRef<HTMLInputElement>(null);

  useLayoutEffect(() => {
    const el = panelRef.current;
    if (!el) return;
    const rect = el.getBoundingClientRect();
    setPosition(computePopoverPosition(anchorRect, rect.width, rect.height));
  }, [anchorRect, attrs.type]);

  useEffect(() => {
    const t = window.setTimeout(() => {
      nameRef.current?.focus();
      nameRef.current?.select();
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
      if (e.key === "Escape") onClose();
    };
    document.addEventListener("mousedown", onMouseDown);
    document.addEventListener("keydown", onKeyDown, true);
    return () => {
      document.removeEventListener("mousedown", onMouseDown);
      document.removeEventListener("keydown", onKeyDown, true);
    };
  }, [onClose]);

  function setField<K extends keyof VarAttrs>(key: K, value: VarAttrs[K]) {
    setAttrs((prev) => ({ ...prev, [key]: value }));
    setError(null);
  }

  function handleDone() {
    const name = attrs.name.trim();
    if (!name || !/^[\w-]+$/.test(name)) {
      setError("Invalid name");
      nameRef.current?.focus();
      return;
    }
    if (attrs.type === "option" && !attrs.options.trim()) {
      setError("Options required");
      return;
    }
    onDone({ ...attrs, name });
  }

  function onFormKeyDown(e: React.KeyboardEvent) {
    if (e.key === "Enter" && !(e.target instanceof HTMLTextAreaElement)) {
      e.preventDefault();
      handleDone();
    }
  }

  return (
    <div
      ref={panelRef}
      className={`var-edit-popover${focused ? " var-edit-popover--active" : ""}`}
      style={{
        top: position?.top ?? anchorRect.top,
        left: position?.left ?? anchorRect.left,
        visibility: position ? "visible" : "hidden",
      }}
      role="dialog"
      aria-label="Edit variable"
      onFocusCapture={() => setFocused(true)}
      onBlurCapture={(e) => {
        if (!panelRef.current?.contains(e.relatedTarget as Node)) {
          setFocused(false);
        }
      }}
    >
      <table className="var-edit-table" onKeyDown={onFormKeyDown}>
        <tbody>
          <Row label="Name">
            <input
              ref={nameRef}
              type="text"
              className="var-edit-field"
              value={attrs.name}
              onChange={(e) => setField("name", e.target.value)}
            />
          </Row>
          <Row label="Type">
            <select
              className="var-edit-field"
              value={attrs.type}
              onChange={(e) => setField("type", e.target.value as VarType)}
            >
              {VAR_TYPES.map((t) => (
                <option key={t} value={t}>
                  {t}
                </option>
              ))}
            </select>
          </Row>
          <Row label="Default">
            <input
              type="text"
              className="var-edit-field"
              value={attrs.value}
              onChange={(e) => setField("value", e.target.value)}
            />
          </Row>
          <Row label="Label">
            <input
              type="text"
              className="var-edit-field"
              value={attrs.label}
              onChange={(e) => setField("label", e.target.value)}
            />
          </Row>
          <Row label="Placeholder">
            <input
              type="text"
              className="var-edit-field"
              value={attrs.placeholder}
              onChange={(e) => setField("placeholder", e.target.value)}
            />
          </Row>
          {attrs.type === "option" && (
            <Row label="Options">
              <input
                type="text"
                className="var-edit-field"
                value={attrs.options}
                placeholder="a,b,c"
                onChange={(e) => setField("options", e.target.value)}
              />
            </Row>
          )}
        </tbody>
      </table>
      {error && <p className="var-edit-error">{error}</p>}
      <div className="var-edit-popover-actions">
        <button type="button" className="var-edit-btn danger" onClick={onDelete}>
          Del
        </button>
        <button type="button" className="var-edit-btn primary" onClick={handleDone}>
          OK
        </button>
      </div>
    </div>
  );
}
