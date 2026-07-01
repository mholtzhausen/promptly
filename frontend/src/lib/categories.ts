import type { CategoryDef } from "../types";

export const DEFAULT_CATEGORY = "general";

export function categoryLabel(slug: string, categories: CategoryDef[]): string {
  const found = categories.find((c) => c.slug === slug);
  if (found) return found.label;
  if (slug === DEFAULT_CATEGORY) return "General";
  return slug;
}

export function categoryChipClass(slug: string, categories: CategoryDef[]): string {
  const found = categories.find((c) => c.slug === slug);
  return found?.chipClass ?? "prompt-category--general";
}

export function allCategoriesSelected(
  selected: Set<string>,
  categories: CategoryDef[],
): boolean {
  return categories.every((c) => selected.has(c.slug));
}

export function initialSelectedCategories(categories: CategoryDef[]): Set<string> {
  return new Set(categories.map((c) => c.slug));
}

export function categorySlugs(categories: CategoryDef[]): string[] {
  return categories.map((c) => c.slug);
}
