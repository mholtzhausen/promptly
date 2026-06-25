import { useEffect, useRef, useState, type RefObject } from "react";
import type { Prompt } from "../types";
import {
  TemplateEditor,
  type TemplateEditorHandle,
} from "./TemplateEditor";

type EditorViewProps = {
  editingPrompt: Prompt | null;
  editorFormRef: RefObject<HTMLFormElement | null>;
  editorError: string | null;
  content: string;
  onContentChange: (value: string) => void;
  onClose: () => void;
  onSave: () => void;
};

export function EditorView({
  editingPrompt,
  editorFormRef,
  editorError,
  content,
  onContentChange,
  onClose,
  onSave,
}: EditorViewProps) {
  const templateEditorRef = useRef<TemplateEditorHandle>(null);
  const [templateFocused, setTemplateFocused] = useState(false);
  const [ctrlHeld, setCtrlHeld] = useState(false);
  const p = editingPrompt;
  const insertExistingMode = templateFocused && ctrlHeld;

  useEffect(() => {
    const isControlKey = (e: KeyboardEvent) =>
      e.key === "Control" ||
      e.code === "ControlLeft" ||
      e.code === "ControlRight";
    const syncCtrl = (e: KeyboardEvent) => {
      if (e.type === "keyup" && isControlKey(e)) {
        setCtrlHeld(false);
        return;
      }
      if (e.type === "keydown" && isControlKey(e)) {
        setCtrlHeld(true);
        return;
      }
      setCtrlHeld(e.ctrlKey);
    };
    const clearCtrl = () => setCtrlHeld(false);
    document.addEventListener("keydown", syncCtrl, true);
    document.addEventListener("keyup", syncCtrl, true);
    window.addEventListener("blur", clearCtrl);
    return () => {
      document.removeEventListener("keydown", syncCtrl, true);
      document.removeEventListener("keyup", syncCtrl, true);
      window.removeEventListener("blur", clearCtrl);
    };
  }, []);

  const handleTemplateFocusChange = (focused: boolean) => {
    setTemplateFocused(focused);
    if (!focused) setCtrlHeld(false);
  };

  useEffect(() => {
    const onKeyDown = (e: KeyboardEvent) => {
      if (e.key !== "Insert" || e.altKey || e.metaKey) return;
      if (document.querySelector(".var-edit-popover, .var-picker-popover")) return;
      e.preventDefault();
      if (e.ctrlKey) {
        templateEditorRef.current?.insertExistingVariable();
      } else {
        templateEditorRef.current?.insertVariable();
      }
    };
    document.addEventListener("keydown", onKeyDown, true);
    return () => document.removeEventListener("keydown", onKeyDown, true);
  }, []);

  return (
    <div className="app editor-view">
      <h1 className="panel-header">
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
            <TemplateEditor
              ref={templateEditorRef}
              value={content}
              onChange={onContentChange}
              onFocusChange={handleTemplateFocusChange}
              onModifierChange={setCtrlHeld}
            />
          </label>
        </form>
      </div>
      <div className="editor-footer panel-footer">
        {editorError && <p className="form-error">{editorError}</p>}
        <div className="buttons">
          <button
            type="button"
            className={
              insertExistingMode ? "insert-var-btn insert-var-btn--existing" : "insert-var-btn"
            }
            title={
              insertExistingMode
                ? "Reuse an existing variable (Ctrl+Ins or Ctrl+click)"
                : "Insert a new variable (Ins or click)"
            }
            onMouseDown={(e) => {
              e.preventDefault();
              if (e.ctrlKey) {
                const rect = e.currentTarget.getBoundingClientRect();
                templateEditorRef.current?.insertExistingVariable(rect);
              } else {
                templateEditorRef.current?.insertVariable();
              }
            }}
          >
            {insertExistingMode ? "Insert Existing Variable" : "Insert Variable"}{" "}
            <kbd className="btn-kbd">{insertExistingMode ? "Ctrl+Ins" : "Ins"}</kbd>
          </button>
          <button type="button" onClick={onClose}>
            Cancel
          </button>
          <button type="button" className="primary" onClick={onSave}>
            {p ? "Update" : "Save"}
          </button>
        </div>
      </div>
    </div>
  );
}
