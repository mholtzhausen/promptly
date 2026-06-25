import {
  forwardRef,
  useCallback,
  useImperativeHandle,
  useMemo,
  useRef,
  useState,
} from "react";
import CodeMirror, { type ReactCodeMirrorRef } from "@uiw/react-codemirror";
import { EditorView } from "@codemirror/view";
import { createVarDecorationsExtension } from "../editor/varDecorations";
import { VarEditPopover } from "./VarEditPopover";
import { VarPickerPopover } from "./VarPickerPopover";
import {
  defaultVarAttrs,
  nextVarName,
  parseVarTag,
  replaceAllVarsWithName,
  serializeVar,
  uniqueVarsByName,
  type VarAttrs,
} from "../lib/templateVars";

function rectFromPos(view: EditorView, pos: number): DOMRect {
  const coords = view.coordsAtPos(pos);
  if (!coords) {
    return new DOMRect(0, 0, 1, 18);
  }
  return new DOMRect(
    coords.left,
    coords.top,
    Math.max(1, coords.right - coords.left),
    Math.max(1, coords.bottom - coords.top),
  );
}

export type TemplateEditorHandle = {
  insertVariable: () => void;
  insertExistingVariable: (anchorRect?: DOMRect) => void;
};

type TemplateEditorProps = {
  value: string;
  onChange: (value: string) => void;
};

type EditState = {
  from: number;
  to: number;
  attrs: VarAttrs;
  originalName: string;
  anchorRect: DOMRect;
};

type PickerState = {
  anchorRect: DOMRect;
};

export const TemplateEditor = forwardRef<TemplateEditorHandle, TemplateEditorProps>(
  function TemplateEditor({ value, onChange }, ref) {
    const editorRef = useRef<ReactCodeMirrorRef>(null);
    const lastCursorPos = useRef(0);
    const [editState, setEditState] = useState<EditState | null>(null);
    const [pickerState, setPickerState] = useState<PickerState | null>(null);
    const onChangeRef = useRef(onChange);
    onChangeRef.current = onChange;

    const openEditor = useCallback(
      (from: number, to: number, anchorRect: DOMRect) => {
        const view = editorRef.current?.view;
        if (!view) return;
        const raw = view.state.doc.sliceString(from, to);
        const parsed = parseVarTag(raw);
        const attrs = parsed ?? defaultVarAttrs("");
        setEditState({
          from,
          to,
          attrs,
          originalName: attrs.name,
          anchorRect,
        });
      },
      [],
    );

    const openEditorFromChip = useCallback(
      (from: number, to: number, anchor: HTMLElement) => {
        openEditor(from, to, anchor.getBoundingClientRect());
      },
      [openEditor],
    );

    const chipClickRef = useRef(openEditorFromChip);
    chipClickRef.current = openEditorFromChip;

    const cursorListener = useMemo(
      () =>
        EditorView.updateListener.of((update) => {
          if (update.selectionSet || update.docChanged) {
            lastCursorPos.current = update.state.selection.main.head;
          }
        }),
      [],
    );

    const extensions = useMemo(
      () => [
        createVarDecorationsExtension((info) =>
          chipClickRef.current(info.from, info.to, info.anchor),
        ),
        cursorListener,
        EditorView.lineWrapping,
        EditorView.theme({
          "&": {
            fontSize: "13px",
            fontFamily: "monospace",
            backgroundColor: "#ffffff",
            border: "1px solid var(--input-border)",
            borderRadius: "8px",
          },
          ".cm-content": {
            minHeight: "120px",
            padding: "8px 10px",
          },
          ".cm-scroller": {
            overflow: "auto",
            fontFamily: "monospace",
          },
          "&.cm-focused": {
            outline: "2px solid var(--input-border-focus)",
            outlineOffset: "-2px",
          },
        }),
      ],
      [cursorListener],
    );

    const applyChange = useCallback((from: number, to: number, insert: string) => {
      const view = editorRef.current?.view;
      if (!view) return;
      view.dispatch({
        changes: { from, to, insert },
      });
      onChangeRef.current(view.state.doc.toString());
    }, []);

    const insertAtCursor = useCallback((tag: string) => {
      const view = editorRef.current?.view;
      if (!view) return;
      const pos = view.hasFocus
        ? view.state.selection.main.head
        : lastCursorPos.current;
      const insertTo = pos + tag.length;
      view.dispatch({
        changes: { from: pos, insert: tag },
        selection: { anchor: insertTo },
      });
      lastCursorPos.current = insertTo;
      onChangeRef.current(view.state.doc.toString());
      view.focus();
    }, []);

    const insertVariable = useCallback(() => {
      const view = editorRef.current?.view;
      if (!view) return;
      const content = view.state.doc.toString();
      const name = nextVarName(content);
      const tag = serializeVar(defaultVarAttrs(name));
      const pos = view.hasFocus
        ? view.state.selection.main.head
        : lastCursorPos.current;
      const insertFrom = pos;
      const insertTo = pos + tag.length;
      view.dispatch({
        changes: { from: pos, insert: tag },
        selection: { anchor: insertTo },
      });
      lastCursorPos.current = insertTo;
      onChangeRef.current(view.state.doc.toString());
      view.focus();
      requestAnimationFrame(() => {
        openEditor(insertFrom, insertTo, rectFromPos(view, insertFrom));
      });
    }, [openEditor]);

    const insertExistingVariable = useCallback((anchorRect?: DOMRect) => {
      const view = editorRef.current?.view;
      if (!view) return;
      const pos = view.hasFocus
        ? view.state.selection.main.head
        : lastCursorPos.current;
      const rect = anchorRect ?? rectFromPos(view, pos);
      setPickerState({ anchorRect: rect });
    }, []);

    const insertExistingVarTag = useCallback(
      (attrs: VarAttrs) => {
        insertAtCursor(serializeVar(attrs));
        setPickerState(null);
      },
      [insertAtCursor],
    );

    useImperativeHandle(
      ref,
      () => ({ insertVariable, insertExistingVariable }),
      [insertVariable, insertExistingVariable],
    );

    return (
      <div className="template-editor">
        <CodeMirror
          ref={editorRef}
          value={value}
          height="100%"
          extensions={extensions}
          onChange={(v) => onChangeRef.current(v)}
          basicSetup={{
            lineNumbers: false,
            foldGutter: false,
            dropCursor: false,
            allowMultipleSelections: false,
            indentOnInput: false,
            bracketMatching: false,
            closeBrackets: false,
            autocompletion: false,
            highlightSelectionMatches: false,
          }}
        />
        {editState && (
          <VarEditPopover
            attrs={editState.attrs}
            anchorRect={editState.anchorRect}
            onClose={() => setEditState(null)}
            onDelete={() => {
              applyChange(editState.from, editState.to, "");
              setEditState(null);
            }}
            onDone={(attrs) => {
              const trimmedName = attrs.name.trim();
              const finalAttrs = { ...attrs, name: trimmedName };
              const view = editorRef.current?.view;
              if (!view) {
                setEditState(null);
                return;
              }
              if (trimmedName === editState.originalName) {
                const content = view.state.doc.toString();
                const updated = replaceAllVarsWithName(
                  content,
                  editState.originalName,
                  finalAttrs,
                );
                view.dispatch({
                  changes: {
                    from: 0,
                    to: content.length,
                    insert: updated,
                  },
                });
                onChangeRef.current(updated);
              } else {
                applyChange(
                  editState.from,
                  editState.to,
                  serializeVar(finalAttrs),
                );
              }
              setEditState(null);
            }}
          />
        )}
        {pickerState && (
          <VarPickerPopover
            variables={uniqueVarsByName(
              editorRef.current?.view?.state.doc.toString() ?? value,
            )}
            anchorRect={pickerState.anchorRect}
            onClose={() => setPickerState(null)}
            onSelect={insertExistingVarTag}
          />
        )}
      </div>
    );
  },
);
