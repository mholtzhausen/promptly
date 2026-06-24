import type { Prompt, VariableDto } from "../types";

type VariablesViewProps = {
  variablePrompt: Prompt;
  variables: VariableDto[];
  preview: string;
  setPreview: (value: string) => void;
  onVariableInput: (name: string, value: string) => void;
  onCancel: () => void;
  onCopy: () => void;
  onCopyAndClose: () => void;
};

export function VariablesView({
  variablePrompt,
  variables,
  preview,
  setPreview,
  onVariableInput,
  onCancel,
  onCopy,
  onCopyAndClose,
}: VariablesViewProps) {
  return (
    <div className="app variables-view">
      <h1 className="panel-header">
        Fill in variables for &lsquo;{variablePrompt.name}&rsquo;
      </h1>
      <div className="variables-body">
        {variables.map((v) => (
          <label key={v.name} className="variable-field">
            <span className="var-name">{v.name}</span>
            {v.description && (
              <span className="var-desc">{v.description}</span>
            )}
            {v.kind === "text" && (
              <input
                type="text"
                defaultValue={v.defaultValue}
                onChange={(e) => onVariableInput(v.name, e.target.value)}
              />
            )}
            {v.kind === "number" && (
              <input
                type="number"
                defaultValue={
                  v.defaultValue ? parseFloat(v.defaultValue) || 0 : 0
                }
                onChange={(e) => onVariableInput(v.name, e.target.value)}
              />
            )}
            {v.kind === "option" && (
              <select
                defaultValue={v.options[0] ?? ""}
                onChange={(e) => onVariableInput(v.name, e.target.value)}
              >
                {v.options.map((opt) => (
                  <option key={opt} value={opt}>
                    {opt}
                  </option>
                ))}
              </select>
            )}
            {v.kind === "multiline" && (
              <textarea
                className="mono multiline"
                defaultValue={v.defaultValue}
                onChange={(e) => onVariableInput(v.name, e.target.value)}
              />
            )}
          </label>
        ))}
        <label className="preview-field">
          Prompt to copy
          <textarea
            className="mono multiline preview"
            value={preview}
            onChange={(e) => setPreview(e.target.value)}
          />
        </label>
      </div>
      <div className="variables-footer panel-footer">
        <div className="buttons">
          <button type="button" onClick={onCancel}>
            Cancel
          </button>
          <button type="button" onClick={onCopy}>
            Copy
          </button>
          <button type="button" className="primary" onClick={onCopyAndClose}>
            Copy &amp; Close
          </button>
        </div>
      </div>
    </div>
  );
}
