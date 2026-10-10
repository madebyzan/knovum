import { shallowRef, watch, type Ref } from "vue";

import type { ImageCanvasSize } from "../../../image-editing/canvas-image-renderer";
import type { ImagePixelRect } from "../../../image-editing/image-edit";
import { imageCanvasHit } from "./image-canvas-hit";

interface ImageBrushSession {
  canDraw: boolean;
  remainingPoints: number;
  brushDiameter: number;
  addStroke: (points: number[]) => void;
}

interface ImageRemovalStrokeInput {
  session: ImageBrushSession;
  canvas: Readonly<Ref<HTMLCanvasElement | null>>;
  size: Readonly<Ref<ImageCanvasSize>>;
  rect: Readonly<Ref<ImagePixelRect | undefined>>;
  enabled: Readonly<Ref<boolean>>;
  frameRotation: Readonly<Ref<number>>;
  redraw: () => void;
}

export function useImageRemovalStrokes(input: ImageRemovalStrokeInput) {
  const session = input.session;
  const draftStroke = shallowRef<number[]>();
  const cursor = shallowRef<{ x: number; y: number }>();
  const pointerId = shallowRef<number>();

  function position(event: PointerEvent) {
    return imageCanvasHit({
      canvas: input.canvas.value,
      size: input.size.value,
      rect: input.rect.value,
      rotation: input.frameRotation.value,
      event,
    });
  }

  function begin(event: PointerEvent) {
    if (!input.enabled.value || !session.canDraw || pointerId.value !== undefined) return;
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
    if (!draft || !hit || draft.length / 2 >= session.remainingPoints) return;
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
    session.addStroke(points.length === 2 ? [...points, ...points] : points);
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
