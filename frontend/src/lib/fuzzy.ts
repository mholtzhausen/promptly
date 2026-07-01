import type { Prompt } from "../types";
import type { CategoryDef } from "../types";

export function fuzzyMatch(text: string, pattern: string): boolean {
  return fuzzyScore(text, pattern) !== null;
}

/** Score how well pattern fuzzy-matches text; higher is better. null if no match. */
export function fuzzyScore(text: string, pattern: string): number | null {
  const pl = pattern.trim().toLowerCase();
  if (!pl) return 0;

  const tl = text.toLowerCase();
  let score = 0;
  let ti = 0;
  let consecutive = 0;
  let prevIdx = -1;

  for (let pi = 0; pi < pl.length; pi++) {
    const idx = tl.indexOf(pl[pi], ti);
    if (idx === -1) return null;

    if (prevIdx >= 0 && idx === prevIdx + 1) {
      consecutive++;
      score += consecutive * 4;
    } else {
      consecutive = 0;
      if (prevIdx >= 0) {
        score -= (idx - prevIdx - 1) * 2;
      } else {
        score -= idx * 3;
      }
    }

    if (idx === 0) {
      score += 8;
    } else {
      const prev = tl[idx - 1];
      if (prev === " " || prev === "-" || prev === "_") {
        score += 5;
      }
    }

    prevIdx = idx;
    ti = idx + 1;
  }

  const subIdx = tl.indexOf(pl);
  if (subIdx !== -1) {
    score += 25;
    if (subIdx === 0) score += 15;
  }

  score -= tl.length * 0.05;
  return score;
}

/** Narrow prompts by selected category slugs. Empty selection yields none. */
export function filterByCategories(
  prompts: Prompt[],
  selectedSlugs: Set<string>,
): Prompt[] {
  if (selectedSlugs.size === 0) return [];
  return prompts.filter((p) => selectedSlugs.has(p.category));
}

export type FilteredPrompts = {
  nameMatches: Prompt[];
  descMatches: Prompt[];
};

export function flattenFilteredPrompts(filtered: FilteredPrompts): Prompt[] {
  return [...filtered.nameMatches, ...filtered.descMatches];
}

/** Filter prompts by fuzzy match: name tier, then description tier; each sorted by closeness. */
export function filterPrompts(
  prompts: Prompt[],
  query: string,
  selectedCategories?: Set<string>,
  allCategorySlugs?: string[],
  _categories: CategoryDef[] = [],
): FilteredPrompts {
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
  if (!q) return { nameMatches: scoped, descMatches: [] };

  const nameMatches: { prompt: Prompt; score: number }[] = [];
  const descMatches: { prompt: Prompt; score: number }[] = [];

  for (const p of scoped) {
    const nameScore = fuzzyScore(p.name, q);
    if (nameScore !== null) {
      nameMatches.push({ prompt: p, score: nameScore });
      continue;
    }
    const descScore = fuzzyScore(p.description, q);
    if (descScore !== null) {
      descMatches.push({ prompt: p, score: descScore });
    }
  }

  const byScore = (
    a: { score: number },
    b: { score: number },
  ) => b.score - a.score;

  nameMatches.sort(byScore);
  descMatches.sort(byScore);

  return {
    nameMatches: nameMatches.map((m) => m.prompt),
    descMatches: descMatches.map((m) => m.prompt),
  };
}

export function filterHistory<T extends { title: string }>(
  entries: T[],
  query: string,
): T[] {
  const q = query.trim();
  if (!q) return entries;
  return entries.filter((e) => fuzzyMatch(e.title, q));
}
