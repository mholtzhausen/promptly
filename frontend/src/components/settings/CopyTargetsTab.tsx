import { useEffect, useRef, useState } from "react";
import type { CopyTarget } from "../../types";

type CopyTargetsTabProps = {
  targets: CopyTarget[];
  lastTarget: string;
  saving: boolean;
  onSave: (targets: CopyTarget[], lastCopyTarget: string) => void;
};

function isValidTarget(row: CopyTarget): boolean {
  const name = row.name.trim();
  const url = row.url.trim();
  return (
    name.length > 0 &&
    (url.startsWith("http://") || url.startsWith("https://"))
  );
}

function normalizeRows(rows: CopyTarget[]): CopyTarget[] {
  return rows.map((row) => ({
    name: row.name.trim(),
    url: row.url.trim(),
  }));
}

export function CopyTargetsTab({
  targets,
  lastTarget,
  saving,
  onSave,
}: CopyTargetsTabProps) {
  const [rows, setRows] = useState<CopyTarget[]>(() =>
    targets.map((t) => ({ ...t })),
  );
  const [defaultTarget, setDefaultTarget] = useState(lastTarget);
  const skipSyncRef = useRef(false);

  useEffect(() => {
    if (skipSyncRef.current) return;
    setRows(targets.map((t) => ({ ...t })));
    setDefaultTarget(lastTarget);
  }, [targets, lastTarget]);

  const persist = (nextRows: CopyTarget[], nextDefault: string) => {
    const hasDrafts = nextRows.some((row) => !isValidTarget(row));
    skipSyncRef.current = hasDrafts;

    setRows(nextRows);
    setDefaultTarget(nextDefault);

    const invalidatingEdit = nextRows.some((row, i) => {
      const prev = rows[i];
      if (!prev) return false;
      return isValidTarget(prev) && !isValidTarget(row);
    });
    if (invalidatingEdit) return;

    const valid = normalizeRows(nextRows).filter(isValidTarget);
    if (valid.length === 0 || valid.length !== nextRows.length) return;

    let resolvedDefault = nextDefault.trim();
    if (!valid.some((row) => row.name === resolvedDefault)) {
      resolvedDefault = valid[0].name;
    }

    onSave(valid, resolvedDefault);
  };

  const updateRow = (index: number, patch: Partial<CopyTarget>) => {
    const next = rows.map((row, i) =>
      i === index ? { ...row, ...patch } : row,
    );
    const updated = next[index];
    let nextDefault = defaultTarget;
    if (
      patch.name !== undefined &&
      defaultTarget === rows[index].name.trim() &&
      updated.name.trim()
    ) {
      nextDefault = updated.name.trim();
    }
    persist(next, nextDefault);
  };

  const addRow = () => {
    skipSyncRef.current = true;
    setRows((prev) => [...prev, { name: "", url: "" }]);
  };

  const removeRow = (index: number) => {
    const removedName = rows[index].name.trim();
    const next = rows.filter((_, i) => i !== index);
    const valid = normalizeRows(next).filter(isValidTarget);
    let nextDefault = defaultTarget;
    if (removedName === defaultTarget) {
      nextDefault = valid[0]?.name ?? defaultTarget;
    }
    persist(next, nextDefault);
  };

  const setDefault = (name: string) => {
    const trimmed = name.trim();
    if (!trimmed) return;
    persist(rows, trimmed);
  };

  return (
    <section aria-label="Copy target settings" className="copy-targets-tab">
      <h2>Copy Targets</h2>
      <p className="settings-hint">
        URLs opened when using Copy &amp; target from the variables view.
      </p>
      <table className="settings-table copy-targets-table">
        <thead>
          <tr>
            <th className="copy-target-col-default" aria-label="Default" />
            <th className="copy-target-col-name">Name</th>
            <th className="copy-target-col-url">URL</th>
            <th className="copy-target-col-actions" aria-label="Actions" />
          </tr>
        </thead>
        <tbody>
          {rows.map((row, index) => {
            const name = row.name.trim();
            const isDefault = name.length > 0 && name === defaultTarget;
            return (
              <tr key={index}>
                <td className="copy-target-col-default">
                  <button
                    type="button"
                    className={
                      isDefault
                        ? "copy-target-default-btn copy-target-default-btn--active"
                        : "copy-target-default-btn"
                    }
                    aria-label={
                      isDefault
                        ? `${name} is the default copy target`
                        : `Set ${name || "target"} as default`
                    }
                    aria-pressed={isDefault}
                    disabled={!name}
                    onClick={() => setDefault(row.name)}
                  >
                    ✓
                  </button>
                </td>
                <td className="copy-target-col-name">
                  <input
                    type="text"
                    className="copy-target-input"
                    value={row.name}
                    onChange={(e) => updateRow(index, { name: e.target.value })}
                    placeholder="claude"
                  />
                </td>
                <td className="copy-target-col-url">
                  <input
                    type="url"
                    className="copy-target-input"
                    value={row.url}
                    onChange={(e) => updateRow(index, { url: e.target.value })}
                    placeholder="https://…"
                  />
                </td>
                <td className="copy-target-col-actions">
                  <button
                    type="button"
                    className="action-btn"
                    aria-label={`Remove ${row.name || "target"}`}
                    onClick={() => removeRow(index)}
                  >
                    ×
                  </button>
                </td>
              </tr>
            );
          })}
        </tbody>
      </table>
      <div className="settings-actions">
        <button type="button" onClick={addRow} disabled={saving}>
          Add target
        </button>
      </div>
      {saving && <p className="settings-status">Saving…</p>}
    </section>
  );
}
