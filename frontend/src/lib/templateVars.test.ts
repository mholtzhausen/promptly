import { describe, expect, it } from "vitest";
import {
  findVarTags,
  parseVarTag,
  serializeVar,
  varChipLabel,
  nextVarName,
} from "./templateVars";

describe("templateVars", () => {
  it("parses a single variable", () => {
    const raw = `<var name="name" type="text" value="world" label="your name" />`;
    const attrs = parseVarTag(raw);
    expect(attrs).toEqual({
      name: "name",
      type: "text",
      value: "world",
      label: "your name",
      placeholder: "",
      options: "",
    });
  });

  it("finds multiple tags", () => {
    const content = `<var name="greeting" type="text" value="Hello" label="Say hi" /> <var name="name" type="text" value="World" />!`;
    const ranges = findVarTags(content);
    expect(ranges).toHaveLength(2);
    expect(ranges[0]!.attrs?.name).toBe("greeting");
    expect(ranges[1]!.attrs?.name).toBe("name");
  });

  it("round-trips serialize and parse", () => {
    const attrs = {
      name: "color",
      type: "option" as const,
      value: "red",
      label: "pick one",
      placeholder: "",
      options: "red,green,blue",
    };
    const raw = serializeVar(attrs);
    expect(parseVarTag(raw)).toEqual(attrs);
  });

  it("handles escaped attributes", () => {
    const raw = `<var name="msg" type="text" value="a&amp;b&quot;c" />`;
    expect(parseVarTag(raw)?.value).toBe('a&b"c');
  });

  it("marks invalid tags", () => {
    const content = `<var type="text" />`;
    const ranges = findVarTags(content);
    expect(ranges[0]!.valid).toBe(false);
    expect(varChipLabel(ranges[0]!.attrs, false)).toBe("invalid variable");
  });

  it("chip label prefers label over name", () => {
    const attrs = parseVarTag(`<var name="x" type="text" label="Hi" />`);
    expect(varChipLabel(attrs, true)).toBe("Hi");
  });

  it("nextVarName increments", () => {
    const content = `<var name="var1" type="text" /> foo <var name="var2" type="text" />`;
    expect(nextVarName(content)).toBe("var3");
  });
});
