import type { CSSProperties } from "react";

export const PASTEL_COLOR_COUNT = 64;
export const PASTEL_GRID_COLS = 8;

export type PastelSwatch = {
  index: number;
  bg: string;
  border: string;
  text: string;
};

/** Legacy CSS class → default pastel index for migration/display. */
const LEGACY_CHIP_TO_PASTEL: Record<string, number> = {
  "prompt-category--development": 0,
  "prompt-category--agents": 8,
  "prompt-category--communication": 16,
  "prompt-category--writing": 24,
  "prompt-category--image": 32,
  "prompt-category--general": 40,
};

function hslToRgb(h: number, s: number, l: number): [number, number, number] {
  const sat = s / 100;
  const light = l / 100;
  const c = (1 - Math.abs(2 * light - 1)) * sat;
  const x = c * (1 - Math.abs(((h / 60) % 2) - 1));
  const m = light - c / 2;
  let r = 0;
  let g = 0;
  let b = 0;
  if (h < 60) [r, g, b] = [c, x, 0];
  else if (h < 120) [r, g, b] = [x, c, 0];
  else if (h < 180) [r, g, b] = [0, c, x];
  else if (h < 240) [r, g, b] = [0, x, c];
  else if (h < 300) [r, g, b] = [x, 0, c];
  else [r, g, b] = [c, 0, x];
  return [
    Math.round((r + m) * 255),
    Math.round((g + m) * 255),
    Math.round((b + m) * 255),
  ];
}

function rgbToHex(r: number, g: number, b: number): string {
  return `#${[r, g, b].map((v) => v.toString(16).padStart(2, "0")).join("")}`;
}

function hslToHex(h: number, s: number, l: number): string {
  const [r, g, b] = hslToRgb(h, s, l);
  return rgbToHex(r, g, b);
}

function buildPastelPalette(): PastelSwatch[] {
  const swatches: PastelSwatch[] = [];
  const cols = PASTEL_GRID_COLS;
  const rows = PASTEL_COLOR_COUNT / cols;
  const topLight = 61;
  const bottomLight = 91;
  for (let row = 0; row < rows; row++) {
    const rowT = rows > 1 ? row / (rows - 1) : 0;
    const light = topLight + rowT * (bottomLight - topLight);
    const sat = 58 - rowT * 10;
    for (let col = 0; col < cols; col++) {
      const hue = (col * 360) / cols;
      const bg = hslToHex(hue, sat, light);
      const border = hslToHex(hue, sat + 4, light - 11);
      const text = hslToHex(hue, sat + 12, light - 34);
      swatches.push({ index: swatches.length, bg, border, text });
    }
  }
  return swatches;
}

export const PASTEL_PALETTE: readonly PastelSwatch[] = buildPastelPalette();

export function pastelChipClass(index: number): string {
  const clamped = ((index % PASTEL_COLOR_COUNT) + PASTEL_COLOR_COUNT) % PASTEL_COLOR_COUNT;
  return `pastel:${clamped}`;
}

export function normalizeChipClass(stored?: string): string {
  if (!stored) return pastelChipClass(0);
  if (stored.startsWith("pastel:")) {
    const index = Number.parseInt(stored.slice(7), 10);
    if (Number.isFinite(index) && index >= 0 && index < PASTEL_COLOR_COUNT) {
      return pastelChipClass(index);
    }
    return pastelChipClass(0);
  }
  const legacy = LEGACY_CHIP_TO_PASTEL[stored];
  if (legacy !== undefined) return pastelChipClass(legacy);
  return pastelChipClass(0);
}

export function chipClassForStored(stored?: string): string {
  return normalizeChipClass(stored);
}

export function pastelIndexFromChipClass(chipClass: string): number {
  const normalized = normalizeChipClass(chipClass);
  return Number.parseInt(normalized.slice(7), 10);
}

export function swatchForChipClass(chipClass: string): PastelSwatch {
  return PASTEL_PALETTE[pastelIndexFromChipClass(chipClass)] ?? PASTEL_PALETTE[0];
}

export function resolveCategoryChip(chipClass: string): {
  className: string;
  style?: CSSProperties;
} {
  const normalized = normalizeChipClass(chipClass);
  if (normalized.startsWith("pastel:")) {
    const swatch = swatchForChipClass(normalized);
    return {
      className: "prompt-category",
      style: {
        color: swatch.text,
        backgroundColor: swatch.bg,
        borderColor: swatch.border,
      },
    };
  }
  return { className: `prompt-category ${normalized}` };
}

export function defaultChipClassForNewCategory(
  existing: Iterable<string>,
): string {
  const used = new Set(
    [...existing].map((chip) => pastelIndexFromChipClass(chip)),
  );
  for (let i = 0; i < PASTEL_COLOR_COUNT; i++) {
    if (!used.has(i)) return pastelChipClass(i);
  }
  return pastelChipClass(0);
}

export function slugFromLabel(label: string): string {
  const slug = label
    .trim()
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, "-")
    .replace(/^-+|-+$/g, "")
    .slice(0, 64);
  return slug || "category";
}

export function uniqueSlugFromLabel(
  label: string,
  taken: Iterable<string>,
  except?: string,
): string {
  const takenSet = new Set(taken);
  const base = slugFromLabel(label);
  let candidate = base;
  let suffix = 2;
  while (takenSet.has(candidate) && candidate !== except) {
    const stem = base.slice(0, Math.max(1, 64 - `-${suffix}`.length));
    candidate = `${stem}-${suffix}`;
    suffix += 1;
  }
  return candidate;
}
