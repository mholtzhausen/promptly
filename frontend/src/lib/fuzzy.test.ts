import { describe, expect, it } from "vitest";
import {
  filterByCategories,
  filterHistory,
  filterPrompts,
  fuzzyMatch,
} from "./fuzzy";
import type { CategoryDef, Prompt } from "../types";

const categories: CategoryDef[] = [
  { slug: "writing", label: "Writing", chipClass: "prompt-category--writing" },
  { slug: "general", label: "General", chipClass: "prompt-category--general" },
];

const sample: Prompt[] = [
  {
    id: 1,
    name: "greet",
    description: "welcomes people",
    content: "Hello",
    category: "writing",
  },
  {
    id: 2,
    name: "farewell",
    description: "goodbye",
    content: "Bye",
    category: "general",
  },
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

  it("includes general prompts when all categories selected", () => {
    const selected = new Set(["writing", "general"]);
    expect(
      filterPrompts(sample, "", selected, ["writing", "general"], categories),
    ).toHaveLength(2);
  });

  it("prioritizes name matches", () => {
    const prompts: Prompt[] = [
      {
        id: 1,
        name: "abc",
        description: "welcoming",
        content: "z",
        category: "general",
      },
      {
        id: 2,
        name: "welcoming",
        description: "x",
        content: "z",
        category: "general",
      },
    ];
    const result = filterPrompts(prompts, "wel");
    expect(result[0].name).toBe("welcoming");
  });

  it("filters by selected categories", () => {
    const prompts: Prompt[] = [
      {
        id: 1,
        name: "a",
        description: "d",
        content: "c",
        category: "development",
      },
      {
        id: 2,
        name: "b",
        description: "d",
        content: "c",
        category: "writing",
      },
    ];
    const selected = new Set(["development"]);
    expect(filterPrompts(prompts, "", selected, ["development", "writing"])).toHaveLength(1);
    expect(filterPrompts(prompts, "", selected)[0].name).toBe("a");
  });

  it("returns none when no categories selected", () => {
    expect(filterByCategories(sample, new Set())).toHaveLength(0);
    expect(filterPrompts(sample, "", new Set())).toHaveLength(0);
  });

  it("matches category labels in search", () => {
    const prompts: Prompt[] = [
      {
        id: 1,
        name: "z",
        description: "d",
        content: "c",
        category: "writing",
      },
    ];
    expect(filterPrompts(prompts, "writ", undefined, undefined, categories)).toHaveLength(1);
  });
});

describe("filterHistory", () => {
  it("filters by title", () => {
    const entries = [{ id: 1, title: "[git](branch:main)", createdAt: 1 }];
    expect(filterHistory(entries, "git")).toHaveLength(1);
  });
});
