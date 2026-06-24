import { describe, expect, it, vi, beforeEach } from "vitest";
import { api } from "./commands";

describe("api commands", () => {
  beforeEach(() => {
    window.ipc = { postMessage: vi.fn() };
  });

  it("listPrompts sends correct IPC envelope", async () => {
    const promise = api.listPrompts();
    expect(window.ipc.postMessage).toHaveBeenCalledTimes(1);
    const raw = (window.ipc.postMessage as ReturnType<typeof vi.fn>).mock
      .calls[0][0] as string;
    const msg = JSON.parse(raw) as { id: string; command: string };
    expect(msg.command).toBe("listPrompts");

    window.__promptlyReceive({
      id: msg.id,
      ok: true,
      data: [{ id: 1, name: "a", description: "d", content: "c" }],
    });

    const result = await promise;
    expect(result).toHaveLength(1);
  });

  it("rejects on IPC error", async () => {
    const promise = api.savePrompt({
      id: null,
      name: "n",
      description: "d",
      content: "c",
    });
    const raw = (window.ipc.postMessage as ReturnType<typeof vi.fn>).mock
      .calls[0][0] as string;
    const msg = JSON.parse(raw) as { id: string };

    window.__promptlyReceive({
      id: msg.id,
      ok: false,
      error: "save failed",
    });

    await expect(promise).rejects.toThrow("save failed");
  });
});
