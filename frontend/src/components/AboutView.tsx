import { useEffect, useState } from "react";
import { api } from "../api/commands";

export type AppInfo = {
  version: string;
  description: string;
  features: string[];
};

type AboutViewProps = {
  onClose: () => void;
};

export function AboutView({ onClose }: AboutViewProps) {
  const [info, setInfo] = useState<AppInfo | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;
    void api
      .getAppInfo()
      .then((data) => {
        if (!cancelled) setInfo(data);
      })
      .catch((err) => {
        if (!cancelled) {
          setError(
            err instanceof Error ? err.message : "Could not load app info.",
          );
        }
      });
    return () => {
      cancelled = true;
    };
  }, []);

  return (
    <div
      className="app about-view"
      onKeyDown={(e) => e.key === "Escape" && onClose()}
    >
      <h1>About Promptly</h1>
      {error ? (
        <p className="confirm-msg">{error}</p>
      ) : info ? (
        <>
          <p className="about-version">Version {info.version}</p>
          <p className="about-description">{info.description}</p>
          <section className="about-features" aria-label="Features">
            <h2>Features</h2>
            <ul>
              {info.features.map((feature) => (
                <li key={feature}>{feature}</li>
              ))}
            </ul>
          </section>
        </>
      ) : (
        <p className="confirm-msg">Loading…</p>
      )}
      <div className="buttons">
        <button type="button" className="primary" onClick={onClose}>
          Close
        </button>
      </div>
    </div>
  );
}
