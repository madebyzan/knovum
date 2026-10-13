import type { VideoEditProject, VideoTransitionKind } from "../project";
import { MIN_CLIP_MS } from "../project";
import { clipEndMs, clipHeadroom, clipIndexOf, clipLengthMs, transitionFrom } from "../timing";
import { commitProject, shiftAfter } from "./commit";

const MAX_TRANSITION_MS = 5_000;
const DEFAULT_TRANSITION_MS = 600;

/** Longest transition the two neighbours can carry; zero when the cut cannot hold one. */
export function maxTransitionMs(project: VideoEditProject, clipId: string) {
  const index = clipIndexOf(project, clipId);
  const clip = project.videoTrack.clips[index];
  const next = project.videoTrack.clips[index + 1];
  if (!clip || !next) return 0;
  const room = Math.min(
    MAX_TRANSITION_MS,
    clipLengthMs(clip) - clipHeadroom(project, clip).incomingMs - MIN_CLIP_MS,
    clipLengthMs(next) - clipHeadroom(project, next).outgoingMs - MIN_CLIP_MS,
  );
  return Math.max(0, room);
}

/**
 * Applies a transition to the cut after `clipId`. The following clips move so the overlap is
 * exactly the transition length, which keeps their relative spacing intact.
 */
export function applyTransition(
  project: VideoEditProject,
  clipId: string,
  kind: VideoTransitionKind,
  durationMs?: number,
): VideoEditProject {
  const room = maxTransitionMs(project, clipId);
  if (room < MIN_CLIP_MS) return project;
  const existing = transitionFrom(project, clipId);
  const length = Math.min(
    room,
    Math.max(MIN_CLIP_MS, durationMs ?? existing?.durationMs ?? DEFAULT_TRANSITION_MS),
  );
  const transitions = existing
    ? project.transitions.map((transition) =>
        transition.id === existing.id ? { ...transition, kind, durationMs: length } : transition,
      )
    : [...project.transitions, { id: crypto.randomUUID(), fromClipId: clipId, kind, durationMs: length }];
  return commitProject(anchorNext({ ...project, transitions }, clipId, length));
}

export function resizeTransition(
  project: VideoEditProject,
  transitionId: string,
  durationMs: number,
): VideoEditProject {
  const transition = project.transitions.find(({ id }) => id === transitionId);
  if (!transition) return project;
  return applyTransition(project, transition.fromClipId, transition.kind, durationMs);
}

export function removeTransition(project: VideoEditProject, transitionId: string): VideoEditProject {
  const transition = project.transitions.find(({ id }) => id === transitionId);
  if (!transition) return project;
  const detached = {
    ...project,
    transitions: project.transitions.filter(({ id }) => id !== transitionId),
  };
  return commitProject(anchorNext(detached, transition.fromClipId, 0));
}

/** Moves the following clip, and everything after it, to the overlap the transition requires. */
function anchorNext(project: VideoEditProject, clipId: string, overlapMs: number): VideoEditProject {
  const index = clipIndexOf(project, clipId);
  const clip = project.videoTrack.clips[index];
  const next = project.videoTrack.clips[index + 1];
  if (!clip || !next) return project;
  const target = clipEndMs(clip) - overlapMs;
  return shiftAfter(project, next.timelineStartMs, target - next.timelineStartMs);
}
