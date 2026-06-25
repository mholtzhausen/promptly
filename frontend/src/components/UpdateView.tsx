export type UpdateDialogPayload = {
  currentVersion: string;
  latestVersion: string;
  changelog: string;
};

type UpdateViewProps = {
  currentVersion: string;
  latestVersion: string;
  changelog: string;
  updating: boolean;
  onClose: () => void;
  onConfirm: () => void;
};

export function UpdateView({
  currentVersion,
  latestVersion,
  changelog,
  updating,
  onClose,
  onConfirm,
}: UpdateViewProps) {
  return (
    <div className="app update-view" onKeyDown={(e) => e.key === "Escape" && !updating && onClose()}>
      <h1>Update Promptly</h1>
      <p className="confirm-msg">
        Upgrade from {currentVersion} to {latestVersion}
      </p>
      <div className="update-changelog" aria-label="Changelog">
        <pre>{changelog}</pre>
      </div>
      <div className="buttons">
        <button type="button" onClick={onClose} disabled={updating}>
          Cancel
        </button>
        <button type="button" className="primary" onClick={onConfirm} disabled={updating}>
          {updating ? "Updating…" : "Update"}
        </button>
      </div>
    </div>
  );
}
