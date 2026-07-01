import { useCallback, useEffect } from "react";
import type { RefObject } from "react";

/** Aggressively focus an input (WebKitGTK show timing). */
export function useAggressiveFocus(
  ref: RefObject<HTMLInputElement | null>,
): () => void {
  return useCallback(() => {
    const attempt = () => ref.current?.focus({ preventScroll: true });
    attempt();
    requestAnimationFrame(attempt);
    window.setTimeout(attempt, 0);
    window.setTimeout(attempt, 50);
    window.setTimeout(attempt, 100);
    window.setTimeout(attempt, 150);
  }, [ref]);
}

/** Keep selected index in bounds when filtered list length changes. */
export function useSelectedIndexBounds(
  length: number,
  selectedIndex: number,
  setSelectedIndex: (index: number) => void,
) {
  useEffect(() => {
    if (length === 0) {
      setSelectedIndex(0);
    } else if (selectedIndex >= length) {
      setSelectedIndex(length - 1);
    }
  }, [length, selectedIndex, setSelectedIndex]);
}

/** Scroll the highlighted row into view. */
export function useScrollSelectedIntoView(
  active: boolean,
  listRef: RefObject<HTMLDivElement | null>,
  selectedIndex: number,
) {
  useEffect(() => {
    if (!active || !listRef.current) return;
    const row = listRef.current.querySelector(
      ".prompt-row.selected",
    ) as HTMLElement | undefined;
    row?.scrollIntoView({ block: "nearest" });
  }, [active, listRef, selectedIndex]);
}

/** Route printable keys to a search input when focus is elsewhere. */
export function usePrintableKeyToInput(
  active: boolean,
  inputRef: RefObject<HTMLInputElement | null>,
  onAppend: (key: string) => void,
) {
  useEffect(() => {
    if (!active) return;
    const onKeyDown = (e: KeyboardEvent) => {
      if (e.target === inputRef.current) return;
      if (e.ctrlKey || e.metaKey || e.altKey) return;
      if (e.key.length !== 1) return;
      e.preventDefault();
      inputRef.current?.focus();
      onAppend(e.key);
    };
    document.addEventListener("keydown", onKeyDown, true);
    return () => document.removeEventListener("keydown", onKeyDown, true);
  }, [active, inputRef, onAppend]);
}
