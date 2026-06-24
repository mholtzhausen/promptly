import { useEffect } from "react";
import { api } from "../api/commands";

/** Debounced live preview via Rust interpolate IPC. */
export function useInterpolatePreview(
  template: string | undefined,
  values: Record<string, string>,
  setPreview: (value: string) => void,
) {
  useEffect(() => {
    if (!template) return;

    const timer = window.setTimeout(() => {
      void (async () => {
        try {
          const result = await api.interpolate({
            template,
            values: Object.entries(values).map(([name, value]) => ({
              name,
              value,
            })),
          });
          setPreview(result);
        } catch {
          // silent
        }
      })();
    }, 100);

    return () => window.clearTimeout(timer);
  }, [template, values, setPreview]);
}
