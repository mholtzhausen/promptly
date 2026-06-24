import type { IpcResponse } from "../types";

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

function newRequestId(): string {
  if (globalThis.crypto?.randomUUID) {
    try {
      return globalThis.crypto.randomUUID();
    } catch {
      // fall through
    }
  }
  requestCounter += 1;
  return `promptly-${Date.now()}-${requestCounter}`;
}

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
