import {
  PASTEL_PALETTE,
  pastelChipClass,
  pastelIndexFromChipClass,
  type PastelSwatch,
} from "../../lib/categoryColors";

type PastelColorPickerProps = {
  value: string;
  onChange: (chipClass: string) => void;
  className?: string;
};

export function PastelColorPicker({
  value,
  onChange,
  className,
}: PastelColorPickerProps) {
  const selected = pastelIndexFromChipClass(value);

  return (
    <div
      className={className ? `pastel-color-picker ${className}` : "pastel-color-picker"}
      role="listbox"
      aria-label="Category color"
    >
      {PASTEL_PALETTE.map((swatch: PastelSwatch) => (
        <button
          key={swatch.index}
          type="button"
          role="option"
          aria-selected={selected === swatch.index}
          aria-label={`Color ${swatch.index + 1}`}
          className={
            selected === swatch.index
              ? "category-color-swatch category-color-swatch--selected"
              : "category-color-swatch"
          }
          style={{
            backgroundColor: swatch.bg,
            borderColor: swatch.border,
          }}
          onClick={() => onChange(pastelChipClass(swatch.index))}
        />
      ))}
    </div>
  );
}
