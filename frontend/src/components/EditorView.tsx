import { useEffect, useRef, type RefObject } from "react";
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
  const p = editingPrompt;

  useEffect(() => {
    const onKeyDown = (e: KeyboardEvent) => {
      if (e.key !== "Insert" || e.ctrlKey || e.metaKey || e.altKey) return;
      if (document.querySelector(".var-edit-popover")) return;
      e.preventDefault();
      templateEditorRef.current?.insertVariable();
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
            />
          </label>
        </form>
      </div>
      <div className="editor-footer panel-footer">
        {editorError && <p className="form-error">{editorError}</p>}
        <div className="buttons">
          <button
            type="button"
            onMouseDown={(e) => {
              e.preventDefault();
              templateEditorRef.current?.insertVariable();
            }}
          >
            Insert Variable <kbd className="btn-kbd">Ins</kbd>
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
