import { onScopeDispose, shallowRef } from "vue";

import type { CanvasPoint } from "../../canvas/contracts/canvas-editor-port";
import type { AnswerPassage } from "../answer-capture";

/** How far the mouse moves before a press on selected words is a drag rather than a click. */
const DRAG_THRESHOLD = 4;

interface PassageDragOptions {
  /** The passage under a press, or nothing when the press is not on selected words. */
  pressed(point: CanvasPoint): AnswerPassage | undefined;
  canvasPointAt(point: CanvasPoint): CanvasPoint | undefined;
  drop(passage: AnswerPassage, point: CanvasPoint): Promise<void>;
  /** A press that never moved, which ends the selection as a click on text does. */
  click(): void;
}

export interface PassageDrag {
  passage: AnswerPassage;
  x: number;
  y: number;
  /** Where on the canvas the passage would land, or nothing while it is over something else. */
  target: CanvasPoint | undefined;
}

/**
 * Drags selected words onto the canvas with the mouse. Not HTML drag and drop: Tauri's native
 * handler takes every drag over the webview to report dropped file paths, so the page never
 * receives `dragover` or `drop`.
 */
export function usePassageDrag(options: PassageDragOptions) {
  const drag = shallowRef<PassageDrag>();
  let session: AbortController | undefined;

  function end() {
    session?.abort();
    session = undefined;
    drag.value = undefined;
  }

  function start(event: MouseEvent) {
    if (event.button !== 0 || session) return;
    const origin = { x: event.clientX, y: event.clientY };
    const passage = options.pressed(origin);
    if (!passage) return;
    // The page owns this press: the browser starts neither its own drag nor a new selection.
    event.preventDefault();
    session = new AbortController();
    const { signal } = session;
    let moved = false;
    const listen = <K extends keyof WindowEventMap>(type: K, listener: (event: WindowEventMap[K]) => void) =>
      globalThis.addEventListener(type, listener, { signal });
    listen("mousemove", (move) => {
      const point = { x: move.clientX, y: move.clientY };
      moved ||= Math.hypot(point.x - origin.x, point.y - origin.y) > DRAG_THRESHOLD;
      if (moved) drag.value = { passage, ...point, target: options.canvasPointAt(point) };
    });
    listen("mouseup", (up) => {
      const target = moved ? options.canvasPointAt({ x: up.clientX, y: up.clientY }) : undefined;
      end();
      if (!moved) options.click();
      else if (target) void options.drop(passage, target);
    });
    listen("keydown", (key) => {
      if (key.key === "Escape") end();
    });
    listen("blur", end);
  }

  onScopeDispose(end);
  return { drag, start };
}
