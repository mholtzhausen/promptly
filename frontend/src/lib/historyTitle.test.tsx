import { describe, expect, it } from "vitest";
import { render, screen } from "@testing-library/react";
import { parseHistoryTitle, HistoryTitleText } from "./historyTitle";

describe("parseHistoryTitle", () => {
  it("parses Rust history title format", () => {
    expect(parseHistoryTitle("[git-commit](branch:main, message:fix)")).toEqual({
      name: "git-commit",
      vars: "branch:main, message:fix",
    });
  });

  it("handles empty vars", () => {
    expect(parseHistoryTitle("[Code Prompt]()")).toEqual({
      name: "Code Prompt",
      vars: null,
    });
  });

  it("returns raw title when format unknown", () => {
    expect(parseHistoryTitle("plain title")).toEqual({
      name: "plain title",
      vars: null,
    });
  });
});

describe("HistoryTitleText", () => {
  it("renders name and vars", () => {
    render(<HistoryTitleText title="[tpl](a:b)" />);
    expect(screen.getByText("tpl")).toBeTruthy();
    expect(screen.getByText(/\(a:b\)/)).toBeTruthy();
  });
});
