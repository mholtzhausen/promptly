import type { Prompt } from "../types";
import type { CategoryDef } from "../types";
import { categoryLabel } from "./categories";
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

/** Narrow prompts by selected category slugs. Empty selection yields none. */
export function filterByCategories(
  prompts: Prompt[],
  selectedSlugs: Set<string>,
): Prompt[] {
  if (selectedSlugs.size === 0) return [];
  return prompts.filter((p) => selectedSlugs.has(p.category));
}

/** Filter prompts by fuzzy match, ordered: name → description → category → content. */
export function filterPrompts(
  prompts: Prompt[],
  query: string,
  selectedCategories?: Set<string>,
  allCategorySlugs?: string[],
  categories: CategoryDef[] = [],
): Prompt[] {
  let scoped = prompts;
  if (selectedCategories) {
    const allSelected =
      allCategorySlugs !== undefined &&
      allCategorySlugs.length > 0 &&
      allCategorySlugs.every((slug) => selectedCategories.has(slug));
    if (!allSelected) {
      scoped = filterByCategories(prompts, selectedCategories);
    }
  }

  const q = query.trim();
  if (!q) return scoped;

  const nameMatches: Prompt[] = [];
  const descMatches: Prompt[] = [];
  const categoryMatches: Prompt[] = [];
  const contentMatches: Prompt[] = [];

  for (const p of scoped) {
    if (fuzzyMatch(p.name, q)) {
      nameMatches.push(p);
    } else if (fuzzyMatch(p.description, q)) {
      descMatches.push(p);
    } else if (fuzzyMatch(categoryLabel(p.category, categories), q)) {
      categoryMatches.push(p);
    } else if (fuzzyMatch(p.content, q)) {
      contentMatches.push(p);
    }
  }

  return [
    ...nameMatches,
    ...descMatches,
    ...categoryMatches,
    ...contentMatches,
  ];
}

export function filterHistory<T extends { title: string }>(
  entries: T[],
  query: string,
): T[] {
  const q = query.trim();
  if (!q) return entries;
  return entries.filter((e) => fuzzyMatch(e.title, q));
}
