import type { Prompt } from "../types";

type DeleteViewProps = {
  deletingPrompt: Prompt;
  onClose: () => void;
  onConfirm: () => void;
};

export function DeleteView({
  deletingPrompt,
  onClose,
  onConfirm,
}: DeleteViewProps) {
  return (
    <div className="app" onKeyDown={(e) => e.key === "Escape" && onClose()}>
      <h1>Delete Prompt Template</h1>
      <p className="confirm-msg">
        Delete &lsquo;{deletingPrompt.name}&rsquo;? This cannot be undone.
      </p>
      <div className="buttons">
        <button type="button" onClick={onClose}>
          Cancel
        </button>
        <button type="button" className="danger" onClick={onConfirm}>
          Delete
        </button>
      </div>
    </div>
  );
}
