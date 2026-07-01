import { resolveCategoryChip } from "../lib/categoryColors";

type CategoryChipProps = {
  label: string;
  chipClass: string;
  className?: string;
};

export function CategoryChip({ label, chipClass, className }: CategoryChipProps) {
  const { className: resolvedClass, style } = resolveCategoryChip(chipClass);
  return (
    <span className={[resolvedClass, className].filter(Boolean).join(" ")} style={style}>
      {label}
    </span>
  );
}
