import { useEffect } from "react";

/** Register Rust host callbacks; cleaned up on unmount. */
export function useHostBridge(options: {
  onShow: () => void;
  focusSearch: () => void;
}) {
  const { onShow, focusSearch } = options;

  useEffect(() => {
    window.__promptlyOnShow = onShow;
    window.__promptlyFocusSearch = focusSearch;
    return () => {
      window.__promptlyOnShow = () => {};
      window.__promptlyFocusSearch = () => {};
    };
  }, [onShow, focusSearch]);
}
