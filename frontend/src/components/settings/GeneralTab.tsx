type GeneralTabProps = {
  seconds: number;
  saving: boolean;
  onSave: (seconds: number) => void;
};

export function GeneralTab({ seconds, saving, onSave }: GeneralTabProps) {
  return (
    <section aria-label="General settings">
      <h2>Notifications</h2>
      <label className="settings-field">
        Notification timeout (seconds)
        <input
          type="number"
          min={1}
          max={60}
          defaultValue={seconds}
          key={seconds}
          onBlur={(e) => {
            const value = Number.parseInt(e.target.value, 10);
            if (Number.isFinite(value) && value >= 1 && value <= 60 && value !== seconds) {
              onSave(value);
            }
          }}
        />
      </label>
      <p className="settings-hint">
        How long ephemeral in-app notifications stay visible (1–60 seconds).
      </p>
      {saving && <p className="settings-status">Saving…</p>}
    </section>
  );
}
