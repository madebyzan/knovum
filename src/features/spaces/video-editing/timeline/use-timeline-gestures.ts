import { shallowRef } from "vue";

import type { VideoEditProject } from "../project";
import type { VideoSelection } from "../selection";
import { projectDurationMs } from "../timing";
import { applyTimelineGesture, type TimelineGesture } from "./gesture-apply";
import { snapTime } from "./snapping";

interface GestureOptions {
  project: () => VideoEditProject;
  timeAt: (event: { clientX: number }) => number;
  pxPerSecond: () => number;
  playheadMs: () => number;
  commit: (project: VideoEditProject, mergeKey?: string) => void;
  seek: (timeMs: number) => void;
  select: (selection: VideoSelection) => void;
  pause: () => void;
}

/**
 * One pointer gesture model for the whole timeline. A gesture produces a live draft project so the
 * lanes show the result while dragging, and only the settled draft enters edit history.
 */
export function useTimelineGestures(options: GestureOptions) {
  const draft = shallowRef<VideoEditProject>();
  const active = shallowRef<TimelineGesture>();
  const snapMs = shallowRef<number>();
  const snapEnabled = shallowRef(true);

  function snapDisabled(event: PointerEvent) {
    return snapEnabled.value ? event.altKey : !event.altKey;
  }

  function snapPoint(
    timeMs: number,
    event: PointerEvent,
    opts?: { exclude?: number[]; includePlayhead?: boolean },
  ) {
    const snapped = snapTime({
      project: options.project(),
      timeMs,
      pxPerSecond: options.pxPerSecond(),
      playheadMs: opts?.includePlayhead === false ? undefined : options.playheadMs(),
      disabled: snapDisabled(event),
      exclude: opts?.exclude,
    });
    snapMs.value = snapped.snappedTo;
    return snapped;
  }

  function snapBlockStart(rawStartMs: number, lengthMs: number, exclude: number[], event: PointerEvent) {
    const clampedStart = Math.max(0, rawStartMs);
    const startSnap = snapPoint(clampedStart, event, { exclude });
    const endSnap = snapPoint(clampedStart + lengthMs, event, { exclude });
    const startDist =
      startSnap.snappedTo !== undefined
        ? Math.abs(startSnap.snappedTo - clampedStart)
        : Number.POSITIVE_INFINITY;
    const endDist =
      endSnap.snappedTo !== undefined
        ? Math.abs(endSnap.snappedTo - (clampedStart + lengthMs))
        : Number.POSITIVE_INFINITY;
    if (endDist < startDist) {
      snapMs.value = endSnap.snappedTo;
      return Math.max(0, endSnap.timeMs - lengthMs);
    }
    snapMs.value = startSnap.snappedTo;
    return startSnap.timeMs;
  }

  const snapper = { snapPoint, snapBlockStart };

  function scrubAt(event: PointerEvent) {
    const durationMs = projectDurationMs(options.project());
    const raw = Math.max(0, Math.min(options.timeAt(event), durationMs));
    const snapped = snapPoint(raw, event, { includePlayhead: false }).timeMs;
    options.seek(Math.max(0, Math.min(snapped, durationMs)));
  }

  function begin(event: PointerEvent, gesture: TimelineGesture) {
    if (event.button !== 0) return;
    event.stopPropagation();
    const target = event.currentTarget as HTMLElement;
    target.setPointerCapture(event.pointerId);
    if (event.pointerType) target.blur();
    options.pause();
    active.value = gesture;
    draft.value = undefined;
    snapMs.value = undefined;
    if (gesture.kind === "scrub") scrubAt(event);
    else if (gesture.kind !== "transition") options.select(selectionOf(gesture));
    else options.select({ kind: "transition", id: gesture.id });
  }

  function move(event: PointerEvent) {
    const gesture = active.value;
    if (!gesture) return;
    if (gesture.kind === "scrub") {
      scrubAt(event);
      return;
    }
    try {
      draft.value = applyTimelineGesture(
        { project: options.project(), pointerMs: options.timeAt(event), event, snapper },
        gesture,
      );
    } catch {
      snapMs.value = undefined;
    }
  }

  function finish() {
    const settled = draft.value;
    const gesture = active.value;
    cancel();
    if (settled && gesture) options.commit(settled, `${gesture.kind}:${gestureId(gesture)}`);
  }

  function cancel() {
    active.value = undefined;
    draft.value = undefined;
    snapMs.value = undefined;
  }

  return {
    draft,
    active,
    snapMs,
    snapEnabled,
    toggleSnap: () => (snapEnabled.value = !snapEnabled.value),
    begin,
    move,
    finish,
    cancel,
  };
}

function selectionOf(gesture: TimelineGesture): VideoSelection {
  if (gesture.kind.startsWith("clip")) return { kind: "clip", id: (gesture as { id: string }).id };
  if (gesture.kind.startsWith("audio")) return { kind: "audio", id: (gesture as { id: string }).id };
  return { kind: "text", id: (gesture as { id: string }).id };
}

function gestureId(gesture: TimelineGesture) {
  return "id" in gesture ? gesture.id : gesture.kind;
}
