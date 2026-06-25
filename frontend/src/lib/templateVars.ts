export type VarType = "text" | "number" | "option" | "multiline";

export interface VarAttrs {
  name: string;
  type: VarType;
  value: string;
  label: string;
  placeholder: string;
  options: string;
}

export interface VarRange {
  from: number;
  to: number;
  raw: string;
  attrs: VarAttrs | null;
  valid: boolean;
}

const VAR_TAG_RE = /<var\s+([^>]*?)\s*\/>/g;

function unescapeAttr(s: string): string {
  return s
    .replace(/&quot;/g, '"')
    .replace(/&lt;/g, "<")
    .replace(/&amp;/g, "&");
}

function escapeAttr(s: string): string {
  return s.replace(/&/g, "&amp;").replace(/"/g, "&quot;").replace(/</g, "&lt;");
}

function parseAttributes(attrsStr: string): Record<string, string> {
  const map: Record<string, string> = {};
  let i = 0;
  while (i < attrsStr.length) {
    while (i < attrsStr.length && /\s/.test(attrsStr[i]!)) i++;
    if (i >= attrsStr.length) break;
    const keyStart = i;
    while (i < attrsStr.length && attrsStr[i] !== "=" && !/\s/.test(attrsStr[i]!)) {
      i++;
    }
    if (i >= attrsStr.length || attrsStr[i] !== "=") break;
    const key = attrsStr.slice(keyStart, i);
    i++;
    while (i < attrsStr.length && /\s/.test(attrsStr[i]!)) i++;
    if (i >= attrsStr.length || attrsStr[i] !== '"') break;
    i++;
    const valStart = i;
    while (i < attrsStr.length) {
      if (attrsStr[i] === "&") {
        i++;
        while (i < attrsStr.length && attrsStr[i] !== ";") i++;
        if (i < attrsStr.length) i++;
      } else if (attrsStr[i] === '"') {
        break;
      } else {
        i++;
      }
    }
    const rawVal = attrsStr.slice(valStart, i);
    if (i < attrsStr.length) i++;
    map[key] = unescapeAttr(rawVal);
  }
  return map;
}

function isValidName(name: string): boolean {
  return /^[\w-]+$/.test(name);
}

function isVarType(s: string): s is VarType {
  return s === "text" || s === "number" || s === "option" || s === "multiline";
}

export function parseVarTag(raw: string): VarAttrs | null {
  const m = raw.match(/^<var\s+([^>]*?)\s*\/>$/);
  if (!m) return null;
  const attrs = parseAttributes(m[1]!);
  const name = (attrs.name ?? "").trim();
  const typeStr = (attrs.type ?? "").trim();
  if (!name || !isValidName(name) || !isVarType(typeStr)) return null;
  return {
    name,
    type: typeStr,
    value: attrs.value ?? "",
    label: attrs.label ?? "",
    placeholder: attrs.placeholder ?? "",
    options: attrs.options ?? "",
  };
}

export function findVarTags(content: string): VarRange[] {
  const ranges: VarRange[] = [];
  const re = new RegExp(VAR_TAG_RE.source, "g");
  let match: RegExpExecArray | null;
  while ((match = re.exec(content)) !== null) {
    const raw = match[0];
    const attrs = parseVarTag(raw);
    ranges.push({
      from: match.index,
      to: match.index + raw.length,
      raw,
      attrs,
      valid: attrs !== null,
    });
  }
  return ranges;
}

export function serializeVar(attrs: VarAttrs): string {
  const parts = [
    `name="${escapeAttr(attrs.name)}"`,
    `type="${escapeAttr(attrs.type)}"`,
  ];
  if (attrs.value) parts.push(`value="${escapeAttr(attrs.value)}"`);
  if (attrs.label) parts.push(`label="${escapeAttr(attrs.label)}"`);
  if (attrs.placeholder) parts.push(`placeholder="${escapeAttr(attrs.placeholder)}"`);
  if (attrs.options) parts.push(`options="${escapeAttr(attrs.options)}"`);
  return `<var ${parts.join(" ")} />`;
}

export function varChipLabel(attrs: VarAttrs | null, valid: boolean): string {
  if (!valid || !attrs) return "invalid variable";
  return attrs.label || attrs.name;
}

export function nextVarName(content: string): string {
  const used = new Set(findVarTags(content).map((r) => r.attrs?.name).filter(Boolean));
  let n = 1;
  while (used.has(`var${n}`)) n++;
  return `var${n}`;
}

export function defaultVarAttrs(name: string): VarAttrs {
  return {
    name,
    type: "text",
    value: "",
    label: "",
    placeholder: "",
    options: "",
  };
}
