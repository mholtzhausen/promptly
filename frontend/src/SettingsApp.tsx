import { useEffect, useState } from "react";
import { api } from "./api/commands";
import type { AppSettings } from "./types";
import { CategoriesTab } from "./components/settings/CategoriesTab";
import { CopyTargetsTab } from "./components/settings/CopyTargetsTab";
import { GeneralTab } from "./components/settings/GeneralTab";

type SettingsTab = "general" | "categories" | "copyTargets";

export function SettingsApp() {
  const [activeTab, setActiveTab] = useState<SettingsTab>("general");
  const [settings, setSettings] = useState<AppSettings | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [saving, setSaving] = useState(false);
  const [categoryEditorOpen, setCategoryEditorOpen] = useState(false);

  useEffect(() => {
    let cancelled = false;
    void api
      .getAppSettings()
      .then((data) => {
        if (!cancelled) setSettings(data);
      })
      .catch((err) => {
        if (!cancelled) {
          setError(
            err instanceof Error ? err.message : "Could not load settings.",
          );
        }
      });
    return () => {
      cancelled = true;
    };
  }, []);

  useEffect(() => {
    const onKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape" && !e.ctrlKey && !categoryEditorOpen) {
        e.preventDefault();
        void api.closeSettingsWindow();
      }
    };
    document.addEventListener("keydown", onKeyDown);
    return () => document.removeEventListener("keydown", onKeyDown);
  }, [categoryEditorOpen]);

  const handleSave = async (
    partial: Parameters<typeof api.saveAppSettings>[0],
  ) => {
    setSaving(true);
    setError(null);
    try {
      const updated = await api.saveAppSettings(partial);
      setSettings(updated);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to save settings.");
    } finally {
      setSaving(false);
    }
  };

  return (
    <div className="app settings-window">
      <h1>Settings</h1>
      {error && <p className="settings-error">{error}</p>}
      <div className="settings-body">
        <nav className="settings-tabs" aria-label="Settings sections">
          <button
            type="button"
            className={`settings-tab${activeTab === "general" ? " settings-tab--active" : ""}`}
            onClick={() => setActiveTab("general")}
          >
            General
          </button>
          <button
            type="button"
            className={`settings-tab${activeTab === "categories" ? " settings-tab--active" : ""}`}
            onClick={() => setActiveTab("categories")}
          >
            Categories
          </button>
          <button
            type="button"
            className={`settings-tab${activeTab === "copyTargets" ? " settings-tab--active" : ""}`}
            onClick={() => setActiveTab("copyTargets")}
          >
            Copy Targets
          </button>
        </nav>
        <div className="settings-panel">
          {!settings ? (
            <p className="confirm-msg">Loading…</p>
          ) : activeTab === "general" ? (
            <GeneralTab
              seconds={settings.ephemeralNotificationSeconds}
              saving={saving}
              onSave={(ephemeralNotificationSeconds) =>
                void handleSave({ ephemeralNotificationSeconds })
              }
            />
          ) : activeTab === "categories" ? (
            <CategoriesTab
              categories={settings.categories}
              saving={saving}
              onSave={(categories) => void handleSave({ categories })}
              onEditorOpenChange={setCategoryEditorOpen}
            />
          ) : (
            <CopyTargetsTab
              targets={settings.targets}
              lastTarget={settings.lastTarget}
              saving={saving}
              onSave={(targets, lastCopyTarget) =>
                void handleSave({ targets, lastCopyTarget })
              }
            />
          )}
        </div>
      </div>
      <div className="settings-footer panel-footer">
        <button type="button" onClick={() => void api.closeSettingsWindow()}>
          Close
        </button>
      </div>
    </div>
  );
}
