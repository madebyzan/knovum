import { getCurrentScope, onScopeDispose, shallowRef } from "vue";

/**
 * One sash: the divider between a sidebar and the work area.
 *
 * The gesture owns the pointer through pointer capture, so it ends on this element no matter where the
 * pointer travels, and it writes the width the caller already owns. Nothing is registered globally and no
 * constraint solver converts the width to another unit and back, which is why the handle cannot drift out
 * of the layout it is dragging. Dragging clamps at the geometry instead of collapsing the sidebar: hiding
 * is the toggle's job, so a gesture never removes the element it is being performed on.
 */

interface SidebarResizeGeometry {
  minSize: number;
  maxSize: number;
}

interface SidebarResizeOptions {
  /** A leading sidebar grows as the pointer moves right, a trailing one as it moves left. */
  edge: "leading" | "trailing";
  geometry: SidebarResizeGeometry;
  width(): number;
  onResize(value: number): void;
  onCommit(): void;
}

const KEYBOARD_STEP = 16;
/** The pointer leaves the handle the moment the drag starts, so the resize cursor becomes the document's. */
const DRAGGING_ATTRIBUTE = "data-sash-dragging";

function setDocumentDragging(active: boolean) {
  document.documentElement.toggleAttribute(DRAGGING_ATTRIBUTE, active);
}

function clampWidth(value: number, geometry: SidebarResizeGeometry) {
  return Math.min(geometry.maxSize, Math.max(geometry.minSize, Math.round(value)));
}

function keyboardWidth(key: string, width: number, geometry: SidebarResizeGeometry, grow: number) {
  if (key === "Home") return geometry.minSize;
  if (key === "End") return geometry.maxSize;
  if (key === "ArrowLeft") return width - KEYBOARD_STEP * grow;
  if (key === "ArrowRight") return width + KEYBOARD_STEP * grow;
  return undefined;
}

export function useSidebarResize(options: SidebarResizeOptions) {
  const dragging = shallowRef(false);
  const grow = options.edge === "leading" ? 1 : -1;
  let originX = 0;
  let originWidth = 0;

  function begin(event: PointerEvent) {
    if (event.button !== 0 || !(event.currentTarget instanceof HTMLElement)) return;
    event.preventDefault();
    event.currentTarget.setPointerCapture(event.pointerId);
    originX = event.clientX;
    originWidth = options.width();
    dragging.value = true;
    setDocumentDragging(true);
  }

  function move(event: PointerEvent) {
    if (!dragging.value) return;
    options.onResize(clampWidth(originWidth + (event.clientX - originX) * grow, options.geometry));
  }

  function end() {
    if (!dragging.value) return;
    dragging.value = false;
    setDocumentDragging(false);
    options.onCommit();
  }

  if (getCurrentScope()) {
    onScopeDispose(() => {
      if (dragging.value) setDocumentDragging(false);
    });
  }

  function keydown(event: KeyboardEvent) {
    const next = keyboardWidth(event.key, options.width(), options.geometry, grow);
    if (next === undefined) return;
    event.preventDefault();
    options.onResize(clampWidth(next, options.geometry));
    options.onCommit();
  }

  return { begin, dragging, end, keydown, move };
}
