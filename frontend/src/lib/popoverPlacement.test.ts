import { describe, expect, it } from "vitest";
import { computePopoverPosition } from "./popoverPlacement";

describe("computePopoverPosition", () => {
  const viewport = { width: 500, height: 400 };

  it("places below anchor when space allows", () => {
    const anchor = new DOMRect(100, 50, 40, 20);
    const pos = computePopoverPosition(anchor, 200, 120, viewport);
    expect(pos.top).toBe(74);
    expect(pos.left).toBe(100);
  });

  it("places above when below overflows", () => {
    const anchor = new DOMRect(100, 350, 40, 20);
    const pos = computePopoverPosition(anchor, 200, 120, viewport);
    expect(pos.top).toBe(226);
  });

  it("overlays anchor when neither above nor below fits", () => {
    const anchor = new DOMRect(100, 10, 40, 380);
    const pos = computePopoverPosition(anchor, 200, 120, viewport);
    expect(pos.top).toBeGreaterThanOrEqual(6);
    expect(pos.top).toBeLessThanOrEqual(274);
  });

  it("clamps horizontal position to viewport", () => {
    const anchor = new DOMRect(450, 50, 40, 20);
    const pos = computePopoverPosition(anchor, 200, 80, viewport);
    expect(pos.left).toBe(294);
  });
});
