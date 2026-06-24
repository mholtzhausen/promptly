import { describe, expect, it } from "vitest";
import { filterHistory, filterPrompts, fuzzyMatch } from "./fuzzy";
import type { Prompt } from "../types";

const sample: Prompt[] = [
  { id: 1, name: "greet", description: "welcomes people", content: "Hello" },
  { id: 2, name: "farewell", description: "goodbye", content: "Bye" },
];

describe("fuzzyMatch", () => {
  it("matches subsequence in order", () => {
    expect(fuzzyMatch("hello world", "hlo")).toBe(true);
    expect(fuzzyMatch("hello", "xyz")).toBe(false);
  });

  it("is case insensitive", () => {
    expect(fuzzyMatch("Hello", "ell")).toBe(true);
  });
});

describe("filterPrompts", () => {
  it("returns all when query empty", () => {
    expect(filterPrompts(sample, "")).toHaveLength(2);
  });

  it("prioritizes name matches", () => {
    const prompts: Prompt[] = [
      { id: 1, name: "abc", description: "welcoming", content: "z" },
      { id: 2, name: "welcome", description: "x", content: "y" },
    ];
    const result = filterPrompts(prompts, "wel");
    expect(result).toHaveLength(2);
    expect(result[0].name).toBe("welcome");
    expect(result[1].name).toBe("abc");
  });
});

describe("filterHistory", () => {
  it("filters by title", () => {
    const entries = [{ id: 1, title: "[git](branch:main)", createdAt: 1 }];
    expect(filterHistory(entries, "git")).toHaveLength(1);
    expect(filterHistory(entries, "nomatch")).toHaveLength(0);
  });
});
