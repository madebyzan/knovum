import { shallowRef, watch, type Ref } from "vue";

import { viewportToLocalPoint } from "../../../canvas/viewport-to-local-point";
import type { ImageCanvasSize } from "../../../image-editing/canvas-image-renderer";
import type { useImageRemovalSession } from "../../../knowledge-derivations/image-removal-session";

interface ImageRect {
  left: number;
  top: number;
  width: number;
  height: number;
}

export function useImageRemovalStrokes(input: {
  canvas: Readonly<Ref<HTMLCanvasElement | null>>;
  size: Readonly<Ref<ImageCanvasSize>>;
  rect: Readonly<Ref<ImageRect | undefined>>;
  enabled: Readonly<Ref<boolean>>;
  frameRotation: Readonly<Ref<number>>;
  session: ReturnType<typeof useImageRemovalSession>;
  redraw: () => void;
}) {
  const draftStroke = shallowRef<number[]>();
  const cursor = shallowRef<{ x: number; y: number }>();
  const pointerId = shallowRef<number>();

  function position(event: PointerEvent) {
    const canvas = input.canvas.value;
    const rect = input.rect.value;
    if (!canvas || !rect) return undefined;
    const local = viewportToLocalPoint(canvas, event.clientX, event.clientY, input.frameRotation.value);
    const x = (local.x * input.size.value.width) / canvas.offsetWidth;
    const y = (local.y * input.size.value.height) / canvas.offsetHeight;
    if (x < rect.left || x > rect.left + rect.width || y < rect.top || y > rect.top + rect.height)
      return undefined;
    return { x, y, point: [(x - rect.left) / rect.width, (y - rect.top) / rect.height] };
  }

  function begin(event: PointerEvent) {
    if (!input.enabled.value || !input.session.canDraw || pointerId.value !== undefined) return;
    const hit = position(event);
    if (!hit) return;
    event.preventDefault();
    input.canvas.value?.setPointerCapture(event.pointerId);
    pointerId.value = event.pointerId;
    draftStroke.value = hit.point;
    input.redraw();
  }

  function move(event: PointerEvent) {
    if (pointerId.value !== undefined && pointerId.value !== event.pointerId) return;
    const hit = position(event);
    cursor.value = hit ? { x: hit.x, y: hit.y } : undefined;
    const draft = draftStroke.value;
    if (!draft || !hit || draft.length / 2 >= input.session.remainingPoints) return;
    event.preventDefault();
    const last = draft.length - 2;
    if (Math.hypot(hit.point[0]! - draft[last]!, hit.point[1]! - draft[last + 1]!) < 0.002) return;
    draft.push(...hit.point);
    input.redraw();
  }

  function commit(event: PointerEvent) {
    if (pointerId.value !== event.pointerId) return;
    move(event);
    pointerId.value = undefined;
    const points = draftStroke.value;
    draftStroke.value = undefined;
    if (!points) return;
    input.session.addStroke(points.length === 2 ? [...points, ...points] : points);
  }

  function cancel(event: PointerEvent) {
    if (pointerId.value !== event.pointerId) return;
    pointerId.value = undefined;
    draftStroke.value = undefined;
    input.redraw();
  }

  watch(input.enabled, (enabled) => {
    if (!enabled) {
      cursor.value = undefined;
      draftStroke.value = undefined;
      pointerId.value = undefined;
    }
    input.redraw();
  });

  return { draftStroke, cursor, begin, move, commit, cancel, leave: () => (cursor.value = undefined) };
}
