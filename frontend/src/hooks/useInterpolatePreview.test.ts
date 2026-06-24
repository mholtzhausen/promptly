import { describe, expect, it, vi, beforeEach, afterEach } from "vitest";
import { act, renderHook } from "@testing-library/react";
import { useInterpolatePreview } from "./useInterpolatePreview";

describe("useInterpolatePreview", () => {
  beforeEach(() => {
    vi.useFakeTimers();
    window.ipc = {
      postMessage: vi.fn(),
    };
  });

  afterEach(() => {
    vi.useRealTimers();
    vi.restoreAllMocks();
  });

  it("debounces interpolate calls", async () => {
    const setPreview = vi.fn();
    let resolveId = "";
    (window.ipc.postMessage as ReturnType<typeof vi.fn>).mockImplementation(
      (msg: string) => {
        const parsed = JSON.parse(msg) as { id: string };
        resolveId = parsed.id;
      },
    );

    const { rerender } = renderHook(
      ({ values }: { values: Record<string, string> }) =>
        useInterpolatePreview("Hello {{name|text||}}", values, setPreview),
      { initialProps: { values: { name: "A" } } },
    );

    rerender({ values: { name: "B" } });
    rerender({ values: { name: "C" } });

    expect(window.ipc.postMessage).not.toHaveBeenCalled();

    await act(async () => {
      await vi.advanceTimersByTimeAsync(100);
    });

    expect(window.ipc.postMessage).toHaveBeenCalledTimes(1);

    await act(async () => {
      window.__promptlyReceive({
        id: resolveId,
        ok: true,
        data: "Hello C",
      });
      await Promise.resolve();
    });

    expect(setPreview).toHaveBeenCalledWith("Hello C");
  });
});
