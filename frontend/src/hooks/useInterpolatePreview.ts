import { useEffect } from "react";
import { api } from "../api/commands";

/** Debounced live preview via Rust interpolate IPC. */
export function useInterpolatePreview(
  template: string | undefined,
  values: Record<string, string>,
  setPreview: (value: string) => void,
  onError?: (message: string) => void,
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
        } catch (err) {
          const msg =
            err instanceof Error && err.message
              ? err.message
              : "Could not update preview.";
          onError?.(msg);
        }
      })();
    }, 100);

    return () => window.clearTimeout(timer);
  }, [template, values, setPreview, onError]);
}
