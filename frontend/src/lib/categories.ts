export type CategorySlug =
  | "development"
  | "agents"
  | "communication"
  | "writing"
  | "image"
  | "general";

export interface CategoryDef {
  slug: CategorySlug;
  label: string;
  chipClass: string;
}

export const CATEGORIES: CategoryDef[] = [
  {
    slug: "development",
    label: "Development",
    chipClass: "prompt-category--development",
  },
  { slug: "agents", label: "Agents", chipClass: "prompt-category--agents" },
  {
    slug: "communication",
    label: "Communication",
    chipClass: "prompt-category--communication",
  },
  { slug: "writing", label: "Writing", chipClass: "prompt-category--writing" },
  { slug: "image", label: "Image", chipClass: "prompt-category--image" },
];

export const FILTERABLE_CATEGORY_SLUGS = CATEGORIES.map((c) => c.slug);

export const DEFAULT_CATEGORY: CategorySlug = "general";

export function categoryLabel(slug: string): string {
  const found = CATEGORIES.find((c) => c.slug === slug);
  if (found) return found.label;
  if (slug === DEFAULT_CATEGORY) return "General";
  return slug;
}

export function isKnownCategory(slug: string): slug is CategorySlug {
  return (
    FILTERABLE_CATEGORY_SLUGS.includes(slug as CategorySlug) ||
    slug === DEFAULT_CATEGORY
  );
}

export function categoryChipClass(slug: string): string {
  const found = CATEGORIES.find((c) => c.slug === slug);
  return found?.chipClass ?? "prompt-category--general";
}

export function allFilterableCategoriesSelected(
  selected: Set<string>,
): boolean {
  return FILTERABLE_CATEGORY_SLUGS.every((slug) => selected.has(slug));
}

export function initialSelectedCategories(): Set<string> {
  return new Set(FILTERABLE_CATEGORY_SLUGS);
}
