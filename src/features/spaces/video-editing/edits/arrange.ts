import type { VideoClip, VideoEditProject } from "../project";
import { clipEndMs, clipLengthMs, compositionAt, canSplitAt, clipIndexOf } from "../timing";
import { clipById, commitProject, shiftAfter } from "./commit";

/** Cuts the clip under the playhead into two contiguous clips. */
export function splitClip(project: VideoEditProject, timeMs: number): VideoEditProject {
  const at = Math.round(timeMs);
  if (!canSplitAt(project, at)) return project;
  const layer = compositionAt(project, at).layers[0]!;
  const { clip, offsetMs } = layer;
  const splitSourceMs = clip.sourceInMs + offsetMs;
  const tail: VideoClip = {
    ...clip,
    id: crypto.randomUUID(),
    sourceInMs: splitSourceMs,
    timelineStartMs: clip.timelineStartMs + offsetMs,
    fadeInMs: 0,
  };
  const clips = project.videoTrack.clips.flatMap((candidate) =>
    candidate.id === clip.id
      ? [{ ...candidate, sourceOutMs: splitSourceMs, fadeOutMs: 0 }, tail]
      : [candidate],
  );
  return commitProject({
    ...project,
    videoTrack: { clips },
    transitions: project.transitions.map((transition) =>
      transition.fromClipId === clip.id ? { ...transition, fromClipId: tail.id } : transition,
    ),
  });
}

/** Removes a clip. Rippling pulls the rest of the timeline into the hole it leaves. */
export function removeClip(
  project: VideoEditProject,
  clipId: string,
  options: { ripple: boolean } = { ripple: true },
): VideoEditProject {
  const index = clipIndexOf(project, clipId);
  const clip = project.videoTrack.clips[index];
  if (!clip || project.videoTrack.clips.length === 1) return project;
  const previous = index > 0 ? project.videoTrack.clips[index - 1] : undefined;
  const next = project.videoTrack.clips[index + 1];
  const detached = {
    ...project,
    videoTrack: { clips: project.videoTrack.clips.filter(({ id }) => id !== clipId) },
    transitions: project.transitions.filter(
      (transition) => transition.fromClipId !== clipId && transition.fromClipId !== previous?.id,
    ),
  };
  if (!options.ripple || !next) return commitProject(detached);
  const anchor = Math.max(clip.timelineStartMs, previous ? clipEndMs(previous) : 0);
  return commitProject(shiftAfter(detached, next.timelineStartMs, anchor - next.timelineStartMs));
}

/** Inserts a copy directly after the clip and rolls the rest of the timeline forward. */
export function duplicateClip(project: VideoEditProject, clipId: string): VideoEditProject {
  const clip = clipById(project, clipId);
  if (!clip) return project;
  const at = clipEndMs(clip);
  const copy: VideoClip = { ...clip, id: crypto.randomUUID(), timelineStartMs: at };
  const rolled = shiftAfter(project, at, clipLengthMs(clip));
  return commitProject({
    ...rolled,
    videoTrack: { clips: [...rolled.videoTrack.clips, copy] },
  });
}

/** Exchanges a clip with its neighbour, keeping the pair's combined span. */
export function swapClip(project: VideoEditProject, clipId: string, direction: -1 | 1): VideoEditProject {
  const index = clipIndexOf(project, clipId);
  const target = index + direction;
  const clips = project.videoTrack.clips;
  if (index < 0 || target < 0 || target >= clips.length) return project;
  const [left, right] = direction === 1 ? [clips[index]!, clips[target]!] : [clips[target]!, clips[index]!];
  const starts = new Map([
    [right.id, left.timelineStartMs],
    [left.id, left.timelineStartMs + clipLengthMs(right)],
  ]);
  const swapped = clips.map((clip) => retimed(clip, starts.get(clip.id)));
  return commitProject({
    ...project,
    videoTrack: { clips: swapped },
    transitions: project.transitions.filter(
      (transition) => transition.fromClipId !== left.id && transition.fromClipId !== right.id,
    ),
  });
}

function retimed(clip: VideoClip, timelineStartMs: number | undefined): VideoClip {
  return timelineStartMs === undefined ? clip : { ...clip, timelineStartMs };
}

/** Free range for dragging a clip along the timeline; a transition anchors that side. */
export function moveBounds(project: VideoEditProject, clipId: string) {
  const index = clipIndexOf(project, clipId);
  const clip = project.videoTrack.clips[index];
  if (!clip) return undefined;
  const previous = index > 0 ? project.videoTrack.clips[index - 1] : undefined;
  const next = project.videoTrack.clips[index + 1];
  const anchored = project.transitions.some(
    (transition) => transition.fromClipId === clipId || transition.fromClipId === previous?.id,
  );
  if (anchored) return undefined;
  return {
    minMs: previous ? clipEndMs(previous) : 0,
    maxMs: next ? next.timelineStartMs - clipLengthMs(clip) : Number.MAX_SAFE_INTEGER,
  };
}

export function moveClip(
  project: VideoEditProject,
  clipId: string,
  timelineStartMs: number,
): VideoEditProject {
  const bounds = moveBounds(project, clipId);
  if (!bounds) return project;
  const start = Math.round(Math.min(Math.max(timelineStartMs, bounds.minMs), bounds.maxMs));
  return commitProject({
    ...project,
    videoTrack: {
      clips: project.videoTrack.clips.map((clip) =>
        clip.id === clipId ? { ...clip, timelineStartMs: start } : clip,
      ),
    },
  });
}

/** Pulls every later clip left so the hole in front of this clip disappears. */
export function closeGapBefore(project: VideoEditProject, clipId: string): VideoEditProject {
  const index = clipIndexOf(project, clipId);
  const clip = project.videoTrack.clips[index];
  if (!clip) return project;
  const previousEnd = index > 0 ? clipEndMs(project.videoTrack.clips[index - 1]!) : 0;
  const gap = clip.timelineStartMs - previousEnd;
  if (gap <= 0) return project;
  return commitProject(shiftAfter(project, clip.timelineStartMs, -gap));
}
