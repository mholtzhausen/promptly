import {
  Decoration,
  EditorView,
  ViewPlugin,
  ViewUpdate,
  type DecorationSet,
} from "@codemirror/view";
import { findVarTags, varChipLabel } from "../lib/templateVars";
import { VarChipWidget, type ChipClickHandler } from "../components/VarChipWidget";

function buildDecorations(
  view: EditorView,
  onChipClick: ChipClickHandler,
): DecorationSet {
  const content = view.state.doc.toString();
  const ranges = findVarTags(content);
  const widgets = ranges.map((r) => {
    const label = varChipLabel(r.attrs, r.valid);
    const typeHint = r.attrs ? `${r.attrs.name} · ${r.attrs.type}` : "malformed";
    return Decoration.replace({
      widget: new VarChipWidget(label, r.valid, typeHint, onChipClick, r.from, r.to),
      inclusive: false,
    }).range(r.from, r.to);
  });
  return Decoration.set(widgets, true);
}

export function createVarDecorationsExtension(
  onChipClick: ChipClickHandler,
): ViewPlugin<{
  decorations: DecorationSet;
  update(update: ViewUpdate): void;
}> {
  return ViewPlugin.fromClass(
    class {
      decorations: DecorationSet;

      constructor(view: EditorView) {
        this.decorations = buildDecorations(view, onChipClick);
      }

      update(update: ViewUpdate) {
        if (update.docChanged || update.viewportChanged) {
          this.decorations = buildDecorations(update.view, onChipClick);
        }
      }
    },
    { decorations: (v) => v.decorations },
  );
}
