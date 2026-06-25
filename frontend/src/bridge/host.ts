import { useEffect } from "react";
import type { UpdateDialogPayload } from "../components/UpdateView";

/** Register Rust host callbacks; cleaned up on unmount. */
export function useHostBridge(options: {
  onShow: () => void;
  focusSearch: () => void;
  onShowUpdateDialog: (payload: UpdateDialogPayload) => void;
  onShowAbout: () => void;
}) {
  const { onShow, focusSearch, onShowUpdateDialog, onShowAbout } = options;

  useEffect(() => {
    window.__promptlyOnShow = onShow;
    window.__promptlyFocusSearch = focusSearch;
    window.__promptlyShowUpdateDialog = onShowUpdateDialog;
    window.__promptlyShowAbout = onShowAbout;
    return () => {
      window.__promptlyOnShow = () => {};
      window.__promptlyFocusSearch = () => {};
      window.__promptlyShowUpdateDialog = () => {};
      window.__promptlyShowAbout = () => {};
    };
  }, [onShow, focusSearch, onShowUpdateDialog, onShowAbout]);
}
