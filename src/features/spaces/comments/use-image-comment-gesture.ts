import { clamp } from "es-toolkit";
import { shallowRef } from "vue";

import { viewportToLocalPoint } from "../canvas/viewport-to-local-point";
import type { SpaceImageComment } from "../model/nodes/node-payload";
import { useImageCommentSessionStore } from "./image-comment-session-store";

export type CommentMark = Pick<SpaceImageComment, "x" | "y" | "shape">;
interface Gesture {
  pointerId: number;
  start: { x: number; y: number };
  points: number[];
}

function point(event: PointerEvent, rotation: number): { x: number; y: number } {
  const element = event.currentTarget as HTMLElement;
  const local = viewportToLocalPoint(element, event.clientX, event.clientY, rotation);
  return {
    x: clamp(local.x / element.offsetWidth, 0, 1),
    y: clamp(local.y / element.offsetHeight, 0, 1),
  };
}

function completeMark(mark: CommentMark | undefined): mark is CommentMark {
  if (mark?.shape?.kind === "stroke") return mark.shape.points.length >= 4;
  if (mark?.shape?.kind === "region") return mark.shape.width >= 0.01 && mark.shape.height >= 0.01;
  return false;
}

export function useImageCommentGesture(complete: (mark: CommentMark) => void, rotation: () => number) {
  const session = useImageCommentSessionStore();
  const gesture = shallowRef<Gesture>();
  const preview = shallowRef<CommentMark>();

  function move(event: PointerEvent) {
    const current = gesture.value;
    if (!current || current.pointerId !== event.pointerId) return;
    event.preventDefault();
    event.stopPropagation();
    const end = point(event, rotation());
    if (session.mode === "region") {
      const x = Math.min(current.start.x, end.x);
      const y = Math.min(current.start.y, end.y);
      preview.value = {
        x: Math.max(current.start.x, end.x),
        y: Math.max(current.start.y, end.y),
        shape: {
          kind: "region",
          x,
          y,
          width: Math.abs(end.x - current.start.x),
          height: Math.abs(end.y - current.start.y),
        },
      };
    } else if (session.mode === "draw") {
      const lastX = current.points.at(-2) ?? current.start.x;
      const lastY = current.points.at(-1) ?? current.start.y;
      if (Math.hypot(end.x - lastX, end.y - lastY) < 0.002 || current.points.length >= 1024) return;
      const points = [...current.points, end.x, end.y];
      gesture.value = { ...current, points };
      preview.value = { x: end.x, y: end.y, shape: { kind: "stroke", points } };
    }
  }

  function start(event: PointerEvent) {
    if (gesture.value) return;
    event.preventDefault();
    event.stopPropagation();
    const origin = point(event, rotation());
    (event.currentTarget as HTMLElement).setPointerCapture(event.pointerId);
    gesture.value = { pointerId: event.pointerId, start: origin, points: [origin.x, origin.y] };
    preview.value = session.mode === "point" ? origin : undefined;
  }

  function finish(event: PointerEvent) {
    const current = gesture.value;
    if (!current || current.pointerId !== event.pointerId) return;
    event.preventDefault();
    event.stopPropagation();
    if (session.mode === "point") complete(current.start);
    else {
      move(event);
      const mark = preview.value;
      if (completeMark(mark)) complete(mark);
    }
    gesture.value = undefined;
    preview.value = undefined;
  }

  function cancel() {
    gesture.value = undefined;
    preview.value = undefined;
  }
  return { preview, start, move, finish, cancel };
}
