import { onScopeDispose, shallowRef } from "vue";

export interface ViewportPoint {
  x: number;
  y: number;
}

/** How far the mouse moves before a press is a drag rather than a click. */
const DRAG_THRESHOLD_PX = 4;

interface PointerDragOptions<Item, Target> {
  /** Where `item` would land under a viewport point, or nothing while it is over anything else. */
  targetAt(point: ViewportPoint): Target | undefined;
  drop(item: Item, target: Target): void | Promise<unknown>;
  /**
   * A press released without moving. The press itself is the page's, so a gesture that replaces a
   * browser default (selecting text, say) restores it here; a button's own click still follows.
   */
  tap?(): void;
}

interface PointerDrag<Item, Target> {
  item: Item;
  x: number;
  y: number;
  target: Target | undefined;
}

/**
 * Drags something with the mouse between surfaces of the window. The desktop shell takes every
 * HTML drag over the webview to report dropped file paths, so the page never receives `dragover`
 * or `drop`: anything dragged inside the window is this gesture instead.
 */
export function usePointerDrag<Item, Target>(options: PointerDragOptions<Item, Target>) {
  const drag = shallowRef<PointerDrag<Item, Target>>();
  let gesture: AbortController | undefined;

  function end() {
    gesture?.abort();
    gesture = undefined;
    drag.value = undefined;
  }

  /** Picks `item` up with the press `event`; presses that pick nothing up never call this. */
  function start(event: MouseEvent, item: Item) {
    if (event.button !== 0 || gesture) return;
    // The browser would start its own drag of the press target, which the shell swallows.
    event.preventDefault();
    gesture = new AbortController();
    const listen = <K extends keyof WindowEventMap>(type: K, listener: (event: WindowEventMap[K]) => void) =>
      globalThis.addEventListener(type, listener, { signal: gesture!.signal, capture: true });
    const origin = { x: event.clientX, y: event.clientY };
    let moved = false;
    listen("mousemove", ({ clientX: x, clientY: y }) => {
      moved ||= Math.hypot(x - origin.x, y - origin.y) > DRAG_THRESHOLD_PX;
      if (moved) drag.value = { item, x, y, target: options.targetAt({ x, y }) };
    });
    listen("mouseup", ({ clientX: x, clientY: y }) => {
      const target = moved ? options.targetAt({ x, y }) : undefined;
      end();
      if (!moved) return options.tap?.();
      swallowNextClick();
      if (target !== undefined) void options.drop(item, target);
    });
    listen("keydown", (key) => {
      if (key.key !== "Escape") return;
      key.stopPropagation();
      end();
    });
    listen("blur", end);
  }

  onScopeDispose(end);
  return { drag, start };
}

/** Whether `element` is what the pointer is over at `point`, not something stacked above it. */
export function isTopmostAt(element: Element | null | undefined, point: ViewportPoint) {
  const hit = element && document.elementFromPoint(point.x, point.y);
  return Boolean(hit && element.contains(hit));
}

function swallowClick(event: MouseEvent) {
  event.preventDefault();
  event.stopPropagation();
}

// A drag released over the press target still clicks it; the drag already did the work. The click
// follows the mouseup in the same task when it comes at all, so the guard ends with that task.
function swallowNextClick() {
  globalThis.addEventListener("click", swallowClick, { capture: true, once: true });
  setTimeout(() => globalThis.removeEventListener("click", swallowClick, { capture: true }));
}
