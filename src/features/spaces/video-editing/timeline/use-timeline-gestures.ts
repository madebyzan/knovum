import { shallowRef } from "vue";

import { moveClip } from "../edits/arrange";
import { moveAudioClip, trimAudioClip } from "../edits/audio";
import { moveTextOverlay, updateTextOverlay } from "../edits/text";
import { resizeTransition } from "../edits/transition";
import { trimClip, type TrimEdge } from "../edits/trim";
import type { VideoEditProject } from "../project";
import type { VideoSelection } from "../selection";
import { clipEndMs } from "../timing";
import { snapTime } from "./snapping";

export type TimelineGesture =
  | { kind: "scrub" }
  | { kind: "clip-move"; id: string; grabMs: number }
  | { kind: "clip-trim"; id: string; edge: TrimEdge }
  | { kind: "audio-move"; id: string; grabMs: number }
  | { kind: "audio-trim"; id: string; edge: TrimEdge }
  | { kind: "text-move"; id: string; grabMs: number }
  | { kind: "text-trim"; id: string; edge: TrimEdge }
  | { kind: "transition"; id: string };

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

  function timeAt(event: PointerEvent) {
    return options.timeAt(event);
  }

  function begin(event: PointerEvent, gesture: TimelineGesture) {
    if (event.button !== 0) return;
    event.stopPropagation();
    (event.currentTarget as HTMLElement).setPointerCapture(event.pointerId);
    options.pause();
    active.value = gesture;
    draft.value = undefined;
    snapMs.value = undefined;
    if (gesture.kind === "scrub") options.seek(snap(event, undefined).timeMs);
    else if (gesture.kind !== "transition") options.select(selectionOf(gesture));
    else options.select({ kind: "transition", id: gesture.id });
  }

  function selectionOf(gesture: TimelineGesture): VideoSelection {
    if (gesture.kind.startsWith("clip")) return { kind: "clip", id: (gesture as { id: string }).id };
    if (gesture.kind.startsWith("audio")) return { kind: "audio", id: (gesture as { id: string }).id };
    return { kind: "text", id: (gesture as { id: string }).id };
  }

  function snap(event: PointerEvent, exclude: readonly number[] | undefined) {
    const result = snapTime({
      project: options.project(),
      timeMs: timeAt(event),
      pxPerSecond: options.pxPerSecond(),
      playheadMs: options.playheadMs(),
      disabled: event.altKey,
      exclude,
    });
    snapMs.value = result.snappedTo;
    return result;
  }

  function move(event: PointerEvent) {
    const gesture = active.value;
    if (!gesture) return;
    const project = options.project();
    if (gesture.kind === "scrub") {
      options.seek(snap(event, undefined).timeMs);
      return;
    }
    try {
      draft.value = apply(project, gesture, event);
    } catch {
      snapMs.value = undefined;
    }
  }

  function apply(project: VideoEditProject, gesture: TimelineGesture, event: PointerEvent) {
    const pointerMs = timeAt(event);
    switch (gesture.kind) {
      case "clip-move":
        return moveClip(project, gesture.id, snap(event, undefined).timeMs - gesture.grabMs);
      case "clip-trim":
        return trimClip(project, gesture.id, gesture.edge, snap(event, undefined).timeMs);
      case "audio-move":
        return moveAudioClip(project, gesture.id, snap(event, undefined).timeMs - gesture.grabMs);
      case "audio-trim":
        return trimAudioClip(project, gesture.id, gesture.edge, snap(event, undefined).timeMs);
      case "text-move":
        return moveTextOverlay(project, gesture.id, snap(event, undefined).timeMs - gesture.grabMs);
      case "text-trim":
        return trimText(project, gesture.id, gesture.edge, snap(event, undefined).timeMs);
      case "transition":
        return stretchTransition(project, gesture.id, pointerMs);
      default:
        return project;
    }
  }

  function finish() {
    const settled = draft.value;
    const gesture = active.value;
    active.value = undefined;
    draft.value = undefined;
    snapMs.value = undefined;
    if (settled && gesture) options.commit(settled, `${gesture.kind}:${gestureId(gesture)}`);
  }

  function cancel() {
    active.value = undefined;
    draft.value = undefined;
    snapMs.value = undefined;
  }

  return { draft, active, snapMs, begin, move, finish, cancel };
}

function gestureId(gesture: TimelineGesture) {
  return "id" in gesture ? gesture.id : gesture.kind;
}

function trimText(
  project: VideoEditProject,
  id: string,
  edge: TrimEdge,
  timelineMs: number,
): VideoEditProject {
  const item = project.textTrack.items.find((candidate) => candidate.id === id);
  if (!item) return project;
  return edge === "start"
    ? updateTextOverlay(project, id, { startMs: Math.min(timelineMs, item.endMs - 100) })
    : updateTextOverlay(project, id, { endMs: Math.max(timelineMs, item.startMs + 100) });
}

/** The transition handle grows symmetrically around the cut it belongs to. */
function stretchTransition(project: VideoEditProject, id: string, pointerMs: number): VideoEditProject {
  const transition = project.transitions.find((candidate) => candidate.id === id);
  const clip = project.videoTrack.clips.find(({ id: clipId }) => clipId === transition?.fromClipId);
  if (!transition || !clip) return project;
  const center = clipEndMs(clip) - transition.durationMs / 2;
  return resizeTransition(project, id, Math.round(Math.abs(pointerMs - center) * 2));
}
