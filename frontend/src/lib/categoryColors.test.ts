import { describe, expect, it } from "vitest";
import {
  PASTEL_COLOR_COUNT,
  normalizeChipClass,
  pastelChipClass,
  slugFromLabel,
  uniqueSlugFromLabel,
} from "./categoryColors";

describe("slugFromLabel", () => {
  it("lowercases and hyphenates", () => {
    expect(slugFromLabel("AI Agents")).toBe("ai-agents");
    expect(slugFromLabel("Development")).toBe("development");
  });

  it("falls back for empty labels", () => {
    expect(slugFromLabel("   ")).toBe("category");
  });
});

describe("uniqueSlugFromLabel", () => {
  it("avoids collisions", () => {
    const taken = ["agents", "agents-2"];
    expect(uniqueSlugFromLabel("Agents", taken)).toBe("agents-3");
  });

  it("allows keeping the same slug when editing", () => {
    expect(uniqueSlugFromLabel("Agents", ["agents"], "agents")).toBe("agents");
  });
});

describe("pastel palette", () => {
  it("has 64 colors", () => {
    expect(PASTEL_COLOR_COUNT).toBe(64);
  });

  it("maps legacy chip classes to pastel ids", () => {
    expect(normalizeChipClass("prompt-category--agents")).toBe(pastelChipClass(8));
  });

  it("round-trips pastel chip class", () => {
    expect(normalizeChipClass("pastel:12")).toBe(pastelChipClass(12));
  });
});
