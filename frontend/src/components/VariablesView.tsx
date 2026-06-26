import { useEffect, useRef, useState } from "react";
import type { CopyTarget } from "../types";
import type { Prompt, VariableDto } from "../types";
import { FormRow } from "./FormRow";

type VariablesViewProps = {
  variablePrompt: Prompt;
  variables: VariableDto[];
  preview: string;
  setPreview: (value: string) => void;
  onVariableInput: (name: string, value: string) => void;
  onCancel: () => void;
  onCopy: () => void;
  copyTargets: CopyTarget[];
  selectedCopyTarget: string;
  onCopyAndOpen: (targetName?: string) => void;
};

function optionDefault(v: VariableDto): string {
  if (v.defaultValue) return v.defaultValue;
  return v.options[0] ?? "";
}

function displayTargetName(name: string): string {
  if (!name) return name;
  return name.charAt(0).toUpperCase() + name.slice(1);
}

export function VariablesView({
  variablePrompt,
  variables,
  preview,
  setPreview,
  onVariableInput,
  onCancel,
  onCopy,
  copyTargets,
  selectedCopyTarget,
  onCopyAndOpen,
}: VariablesViewProps) {
  const [copyMenuOpen, setCopyMenuOpen] = useState(false);
  const copyMenuRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!copyMenuOpen) return;
    const onPointerDown = (e: MouseEvent) => {
      if (
        copyMenuRef.current &&
        !copyMenuRef.current.contains(e.target as Node)
      ) {
        setCopyMenuOpen(false);
      }
    };
    document.addEventListener("mousedown", onPointerDown);
    return () => document.removeEventListener("mousedown", onPointerDown);
  }, [copyMenuOpen]);

  const selectedLabel = displayTargetName(selectedCopyTarget);

  return (
    <div className="app variables-view">
      <h1 className="panel-header">
        Fill in variables for &lsquo;{variablePrompt.name}&rsquo;
      </h1>
      <div className="variables-body">
        {variables.length > 0 && (
          <table className="compact-form-table variables-fields-table">
            <tbody>
              {variables.map((v) => (
                <FormRow key={v.name} label={v.label || v.name}>
                  {v.kind === "text" && (
                    <input
                      type="text"
                      defaultValue={v.defaultValue}
                      placeholder={v.placeholder || undefined}
                      onChange={(e) => onVariableInput(v.name, e.target.value)}
                    />
                  )}
                  {v.kind === "number" && (
                    <input
                      type="number"
                      defaultValue={
                        v.defaultValue ? parseFloat(v.defaultValue) || 0 : 0
                      }
                      placeholder={v.placeholder || undefined}
                      onChange={(e) => onVariableInput(v.name, e.target.value)}
                    />
                  )}
                  {v.kind === "option" && (
                    <select
                      defaultValue={optionDefault(v)}
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
                      placeholder={v.placeholder || undefined}
                      onChange={(e) => onVariableInput(v.name, e.target.value)}
                    />
                  )}
                </FormRow>
              ))}
            </tbody>
          </table>
        )}
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
          <div className="copy-target-split" ref={copyMenuRef}>
            {copyMenuOpen && copyTargets.length > 0 && (
              <div className="copy-target-menu" role="menu">
                {copyTargets.map((target) => (
                  <button
                    key={target.name}
                    type="button"
                    role="menuitem"
                    aria-current={
                      target.name === selectedCopyTarget ? "true" : undefined
                    }
                    onClick={() => {
                      setCopyMenuOpen(false);
                      onCopyAndOpen(target.name);
                    }}
                  >
                    {displayTargetName(target.name)}
                  </button>
                ))}
              </div>
            )}
            <button
              type="button"
              className="primary copy-target-main"
              onClick={() => onCopyAndOpen()}
            >
              Copy &amp; {selectedLabel}
            </button>
            <button
              type="button"
              className="primary copy-target-toggle"
              aria-expanded={copyMenuOpen}
              aria-haspopup="menu"
              aria-label="Choose copy target"
              onClick={() => setCopyMenuOpen((open) => !open)}
            >
              ▲
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}
