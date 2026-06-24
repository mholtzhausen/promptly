import type { IpcResponse } from "./types";

// Wry exposes window.ipc.postMessage to send JSON to the Rust host.
declare global {
  interface Window {
    ipc: { postMessage(message: string): void };
    __promptlyReceive: (response: IpcResponse<unknown>) => void;
    __promptlyOnShow: () => void;
    __promptlyFocusSearch: () => void;
  }
}

interface PendingRequest {
  resolve: (value: unknown) => void;
  reject: (reason: unknown) => void;
}

const pending = new Map<string, PendingRequest>();
let requestCounter = 0;

/** WebKitGTK loads `with_html` at null origin — not always a secure context for `crypto.randomUUID`. */
function newRequestId(): string {
  if (globalThis.crypto?.randomUUID) {
    try {
      return globalThis.crypto.randomUUID();
    } catch {
      // fall through to deterministic fallback
    }
  }
  requestCounter += 1;
  return `promptly-${Date.now()}-${requestCounter}`;
}

/**
 * Called by Rust after script evaluation: resolves the matching pending request.
 */
window.__promptlyReceive = (response: IpcResponse<unknown>) => {
  const entry = pending.get(response.id);
  if (!entry) return;
  pending.delete(response.id);
  if (response.ok) {
    entry.resolve(response.data);
  } else {
    entry.reject(new Error(response.error));
  }
};

/**
 * Send an IPC command to the Rust backend and return the typed response.
 */
export function request<T>(command: string, payload?: unknown): Promise<T> {
  return new Promise<T>((resolve, reject) => {
    const id = newRequestId();
    pending.set(id, { resolve: resolve as (v: unknown) => void, reject });
    try {
      window.ipc.postMessage(JSON.stringify({ id, command, payload }));
    } catch (err) {
      pending.delete(id);
      reject(err instanceof Error ? err : new Error("IPC unavailable"));
    }
  });
}
