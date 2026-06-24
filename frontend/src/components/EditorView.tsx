import type { RefObject } from "react";
import type { Prompt } from "../types";

type EditorViewProps = {
  editingPrompt: Prompt | null;
  editorFormRef: RefObject<HTMLFormElement | null>;
  editorError: string | null;
  onClose: () => void;
  onSave: () => void;
};

export function EditorView({
  editingPrompt,
  editorFormRef,
  editorError,
  onClose,
  onSave,
}: EditorViewProps) {
  const p = editingPrompt;
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
            <textarea
              name="content"
              className="mono"
              defaultValue={p?.content ?? ""}
            />
          </label>
        </form>
      </div>
      <div className="editor-footer panel-footer">
        <p className="help">
          Use {"{{name|type|default|desc}}"} placeholders. Types: text, number,
          option, multiline.
        </p>
        {editorError && <p className="form-error">{editorError}</p>}
        <div className="buttons">
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
