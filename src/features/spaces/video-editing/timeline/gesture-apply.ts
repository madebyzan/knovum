import { moveClip } from "../edits/arrange";
import { moveAudioClip, trimAudioClip } from "../edits/audio";
import { moveTextOverlay, updateTextOverlay } from "../edits/text";
import { resizeTransition } from "../edits/transition";
import { trimClip, type TrimEdge } from "../edits/trim";
import type { VideoEditProject } from "../project";
import { clipEndMs, clipLengthMs } from "../timing";

export type TimelineGesture =
  | { kind: "scrub" }
  | { kind: "clip-move"; id: string; grabMs: number }
  | { kind: "clip-trim"; id: string; edge: TrimEdge }
  | { kind: "audio-move"; id: string; grabMs: number }
  | { kind: "audio-trim"; id: string; edge: TrimEdge }
  | { kind: "text-move"; id: string; grabMs: number }
  | { kind: "text-trim"; id: string; edge: TrimEdge }
  | { kind: "transition"; id: string };

interface GestureSnapper {
  snapPoint: (
    timeMs: number,
    event: PointerEvent,
    opts?: { exclude?: number[]; includePlayhead?: boolean },
  ) => { timeMs: number };
  snapBlockStart: (rawStartMs: number, lengthMs: number, exclude: number[], event: PointerEvent) => number;
}

export interface GestureApplyContext {
  project: VideoEditProject;
  pointerMs: number;
  event: PointerEvent;
  snapper: GestureSnapper;
}

function applyMoveGesture(
  ctx: GestureApplyContext,
  gesture: Extract<TimelineGesture, { kind: "clip-move" | "audio-move" | "text-move" }>,
): VideoEditProject {
  const { project, pointerMs, event, snapper } = ctx;
  if (gesture.kind === "clip-move") {
    const clip = project.videoTrack.clips.find((c) => c.id === gesture.id);
    if (!clip) return project;
    const start = snapper.snapBlockStart(
      pointerMs - gesture.grabMs,
      clipLengthMs(clip),
      [clip.timelineStartMs, clipEndMs(clip)],
      event,
    );
    return moveClip(project, gesture.id, start);
  }
  if (gesture.kind === "audio-move") {
    const clip = project.audioTrack.clips.find((c) => c.id === gesture.id);
    if (!clip) return project;
    const start = snapper.snapBlockStart(
      pointerMs - gesture.grabMs,
      clipLengthMs(clip),
      [clip.timelineStartMs, clipEndMs(clip)],
      event,
    );
    return moveAudioClip(project, gesture.id, start);
  }
  const item = project.textTrack.items.find((c) => c.id === gesture.id);
  if (!item) return project;
  const start = snapper.snapBlockStart(
    pointerMs - gesture.grabMs,
    item.endMs - item.startMs,
    [item.startMs, item.endMs],
    event,
  );
  return moveTextOverlay(project, gesture.id, start);
}

function applyTrimGesture(
  ctx: GestureApplyContext,
  gesture: Extract<TimelineGesture, { kind: "clip-trim" | "audio-trim" | "text-trim" }>,
): VideoEditProject {
  const { project, pointerMs, event, snapper } = ctx;
  if (gesture.kind === "clip-trim") {
    const clip = project.videoTrack.clips.find((c) => c.id === gesture.id);
    const exclude = clip ? [gesture.edge === "start" ? clip.timelineStartMs : clipEndMs(clip)] : [];
    return trimClip(
      project,
      gesture.id,
      gesture.edge,
      snapper.snapPoint(pointerMs, event, { exclude }).timeMs,
    );
  }
  if (gesture.kind === "audio-trim") {
    const clip = project.audioTrack.clips.find((c) => c.id === gesture.id);
    const exclude = clip ? [gesture.edge === "start" ? clip.timelineStartMs : clipEndMs(clip)] : [];
    return trimAudioClip(
      project,
      gesture.id,
      gesture.edge,
      snapper.snapPoint(pointerMs, event, { exclude }).timeMs,
    );
  }
  const item = project.textTrack.items.find((c) => c.id === gesture.id);
  const exclude = item ? [gesture.edge === "start" ? item.startMs : item.endMs] : [];
  return trimText(project, gesture.id, gesture.edge, snapper.snapPoint(pointerMs, event, { exclude }).timeMs);
}

export function applyTimelineGesture(ctx: GestureApplyContext, gesture: TimelineGesture): VideoEditProject {
  if (gesture.kind === "clip-move" || gesture.kind === "audio-move" || gesture.kind === "text-move") {
    return applyMoveGesture(ctx, gesture);
  }
  if (gesture.kind === "clip-trim" || gesture.kind === "audio-trim" || gesture.kind === "text-trim") {
    return applyTrimGesture(ctx, gesture);
  }
  if (gesture.kind === "transition") return stretchTransition(ctx.project, gesture.id, ctx.pointerMs);
  return ctx.project;
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

function stretchTransition(project: VideoEditProject, id: string, pointerMs: number): VideoEditProject {
  const transition = project.transitions.find((candidate) => candidate.id === id);
  const clip = project.videoTrack.clips.find(({ id: clipId }) => clipId === transition?.fromClipId);
  if (!transition || !clip) return project;
  const center = clipEndMs(clip) - transition.durationMs / 2;
  return resizeTransition(project, id, Math.round(Math.abs(pointerMs - center) * 2));
}
