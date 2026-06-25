import { WidgetType } from "@codemirror/view";

export type ChipClickHandler = (info: {
  from: number;
  to: number;
  anchor: HTMLElement;
}) => void;

export class VarChipWidget extends WidgetType {
  constructor(
    private label: string,
    private valid: boolean,
    private typeHint: string,
    private onClick: ChipClickHandler,
    private from: number,
    private to: number,
  ) {
    super();
  }

  eq(other: VarChipWidget): boolean {
    return (
      this.label === other.label &&
      this.valid === other.valid &&
      this.from === other.from &&
      this.to === other.to
    );
  }

  toDOM(): HTMLElement {
    const span = document.createElement("span");
    span.className = this.valid ? "var-chip" : "var-chip var-chip--invalid";
    span.textContent = this.label;
    span.title = this.typeHint;
    span.setAttribute("role", "button");
    span.setAttribute("tabindex", "0");
    const from = this.from;
    const to = this.to;
    const onClick = this.onClick;
    span.addEventListener("mousedown", (e) => {
      e.preventDefault();
      e.stopPropagation();
    });
    span.addEventListener("click", (e) => {
      e.preventDefault();
      e.stopPropagation();
      onClick({ from, to, anchor: span });
    });
    span.addEventListener("keydown", (e) => {
      if (e.key === "Enter" || e.key === " ") {
        e.preventDefault();
        onClick({ from, to, anchor: span });
      }
    });
    return span;
  }

  ignoreEvent(): boolean {
    return true;
  }
}
