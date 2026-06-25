const MARGIN = 6;
const GAP = 4;

export function computePopoverPosition(
  anchor: DOMRect,
  popoverWidth: number,
  popoverHeight: number,
  viewport = { width: window.innerWidth, height: window.innerHeight },
): { top: number; left: number } {
  let left = anchor.left;
  if (left + popoverWidth + MARGIN > viewport.width) {
    left = Math.max(MARGIN, viewport.width - popoverWidth - MARGIN);
  }
  if (left < MARGIN) {
    left = MARGIN;
  }

  const below = anchor.bottom + GAP;
  const above = anchor.top - GAP - popoverHeight;
  const over = anchor.top + (anchor.height - popoverHeight) / 2;

  let top: number;
  if (below + popoverHeight + MARGIN <= viewport.height) {
    top = below;
  } else if (above >= MARGIN) {
    top = above;
  } else {
    top = over;
    if (top + popoverHeight + MARGIN > viewport.height) {
      top = Math.max(MARGIN, viewport.height - popoverHeight - MARGIN);
    }
    if (top < MARGIN) {
      top = MARGIN;
    }
  }

  return { top, left };
}
