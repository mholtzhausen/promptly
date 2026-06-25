import { describe, expect, it } from "vitest";
import {
  findVarTags,
  parseVarTag,
  replaceAllVarsWithName,
  serializeVar,
  uniqueVarsByName,
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

  it("uniqueVarsByName deduplicates by name", () => {
    const content = `<var name="a" type="text" label="A" /> mid <var name="b" type="number" /> <var name="a" type="text" label="other" />`;
    const unique = uniqueVarsByName(content);
    expect(unique).toHaveLength(2);
    expect(unique[0]!.name).toBe("a");
    expect(unique[0]!.label).toBe("A");
    expect(unique[1]!.name).toBe("b");
  });

  it("replaceAllVarsWithName syncs every reference", () => {
    const content = `Hi <var name="n" type="text" value="old" /> and <var name="n" type="text" value="stale" />`;
    const updated = replaceAllVarsWithName(content, "n", {
      name: "n",
      type: "text",
      value: "new",
      label: "Name",
      placeholder: "",
      options: "",
    });
    const tags = findVarTags(updated);
    expect(tags).toHaveLength(2);
    expect(tags.every((t) => t.attrs?.value === "new")).toBe(true);
    expect(tags.every((t) => t.attrs?.label === "Name")).toBe(true);
  });
});
