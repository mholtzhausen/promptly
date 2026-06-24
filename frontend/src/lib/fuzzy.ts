import type { Prompt } from "../types";

/** Fuzzy subsequence match — characters of pattern must appear in order in text. */
export function fuzzyMatch(text: string, pattern: string): boolean {
  const tl = text.toLowerCase();
  const pl = pattern.toLowerCase();
  let ti = 0;
  for (let pi = 0; pi < pl.length; pi++) {
    const idx = tl.indexOf(pl[pi], ti);
    if (idx === -1) return false;
    ti = idx + 1;
  }
  return true;
}

/** Filter prompts by fuzzy match, ordered: name → description → content. */
export function filterPrompts(prompts: Prompt[], query: string): Prompt[] {
  const q = query.trim();
  if (!q) return prompts;

  const nameMatches: Prompt[] = [];
  const descMatches: Prompt[] = [];
  const contentMatches: Prompt[] = [];

  for (const p of prompts) {
    if (fuzzyMatch(p.name, q)) {
      nameMatches.push(p);
    } else if (fuzzyMatch(p.description, q)) {
      descMatches.push(p);
    } else if (fuzzyMatch(p.content, q)) {
      contentMatches.push(p);
    }
  }

  return [...nameMatches, ...descMatches, ...contentMatches];
}

export function filterHistory<T extends { title: string }>(
  entries: T[],
  query: string,
): T[] {
  const q = query.trim();
  if (!q) return entries;
  return entries.filter((e) => fuzzyMatch(e.title, q));
}
