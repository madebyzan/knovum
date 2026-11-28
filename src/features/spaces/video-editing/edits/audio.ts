import type { VideoAudioClip, VideoEditProject } from "../project";
import { MIN_CLIP_MS } from "../project";
import { clipEndMs, clipLengthMs, projectDurationMs } from "../timing";
import { assetOf, commitProject } from "./commit";
import type { TrimEdge, TrimBounds } from "./trim";

function audioClipById(project: VideoEditProject, clipId: string) {
  return project.audioTrack.clips.find(({ id }) => id === clipId);
}

/** Lane neighbours and the project bounds decide how far an audio clip can travel. */
function audioMoveBounds(project: VideoEditProject, clipId: string): TrimBounds | undefined {
  const clip = audioClipById(project, clipId);
  if (!clip) return undefined;
  const lane = project.audioTrack.clips
    .filter((candidate) => candidate.lane === clip.lane && candidate.id !== clipId)
    .toSorted((left, right) => left.timelineStartMs - right.timelineStartMs);
  const previous = lane.findLast((candidate) => clipEndMs(candidate) <= clip.timelineStartMs);
  const next = lane.find((candidate) => candidate.timelineStartMs >= clipEndMs(clip));
  return {
    minMs: previous ? clipEndMs(previous) : 0,
    maxMs:
      Math.min(next ? next.timelineStartMs : Number.MAX_SAFE_INTEGER, projectDurationMs(project)) -
      clipLengthMs(clip),
  };
}

export function moveAudioClip(
  project: VideoEditProject,
  clipId: string,
  timelineStartMs: number,
): VideoEditProject {
  const bounds = audioMoveBounds(project, clipId);
  const clip = audioClipById(project, clipId);
  if (!bounds || !clip || bounds.maxMs < bounds.minMs) return project;
  const start = Math.round(Math.min(Math.max(timelineStartMs, bounds.minMs), bounds.maxMs));
  return replace(project, { ...clip, timelineStartMs: start });
}

function audioTrimBounds(project: VideoEditProject, clipId: string, edge: TrimEdge): TrimBounds | undefined {
  const clip = audioClipById(project, clipId);
  const asset = clip && assetOf(project, clip.assetId);
  if (!clip || !asset) return undefined;
  const travel = audioMoveBounds(project, clipId);
  const minimum = Math.max(MIN_CLIP_MS, clip.fadeInMs + clip.fadeOutMs);
  if (edge === "start")
    return {
      minMs: Math.max(travel?.minMs ?? 0, clip.timelineStartMs - clip.sourceInMs),
      maxMs: clipEndMs(clip) - minimum,
    };
  return {
    minMs: clip.timelineStartMs + minimum,
    maxMs: Math.min(
      clip.timelineStartMs + (asset.durationMs - clip.sourceInMs),
      projectDurationMs(project),
      (travel?.maxMs ?? Number.MAX_SAFE_INTEGER) + clipLengthMs(clip),
    ),
  };
}

export function trimAudioClip(
  project: VideoEditProject,
  clipId: string,
  edge: TrimEdge,
  timelineMs: number,
): VideoEditProject {
  const bounds = audioTrimBounds(project, clipId, edge);
  const clip = audioClipById(project, clipId);
  if (!bounds || !clip) return project;
  const position = Math.round(Math.min(Math.max(timelineMs, bounds.minMs), bounds.maxMs));
  if (position === (edge === "start" ? clip.timelineStartMs : clipEndMs(clip))) return project;
  return replace(
    project,
    edge === "start"
      ? {
          ...clip,
          sourceInMs: clip.sourceInMs + (position - clip.timelineStartMs),
          timelineStartMs: position,
        }
      : { ...clip, sourceOutMs: clip.sourceOutMs + (position - clipEndMs(clip)) },
  );
}

export function setAudioClipGain(
  project: VideoEditProject,
  clipId: string,
  patch: { volume?: number; fadeInMs?: number; fadeOutMs?: number },
): VideoEditProject {
  const clip = audioClipById(project, clipId);
  if (!clip) return project;
  const next = { ...clip, ...patch };
  if (next.fadeInMs + next.fadeOutMs > clipLengthMs(next)) return project;
  return replace(project, next);
}

/** Moves an audio clip to another lane, keeping its position when the lane has room. */
export function setAudioClipLane(project: VideoEditProject, clipId: string, lane: number): VideoEditProject {
  const clip = audioClipById(project, clipId);
  if (!clip || clip.lane === lane) return project;
  const blocked = project.audioTrack.clips.some(
    (candidate) =>
      candidate.lane === lane &&
      candidate.timelineStartMs < clipEndMs(clip) &&
      clip.timelineStartMs < clipEndMs(candidate),
  );
  return blocked ? project : replace(project, { ...clip, lane });
}

export function canSplitAudioAt(clip: VideoAudioClip, atMs: number): boolean {
  const at = Math.round(atMs);
  return at >= clip.timelineStartMs + MIN_CLIP_MS && at <= clipEndMs(clip) - MIN_CLIP_MS;
}

export function splitAudioClip(project: VideoEditProject, clipId: string, atMs: number): VideoEditProject {
  const clip = audioClipById(project, clipId);
  const at = Math.round(atMs);
  if (!clip || !canSplitAudioAt(clip, at)) return project;
  const offsetMs = at - clip.timelineStartMs;
  const splitSourceMs = clip.sourceInMs + offsetMs;
  const head: VideoAudioClip = {
    ...clip,
    sourceOutMs: splitSourceMs,
    fadeOutMs: 0,
    fadeInMs: Math.min(clip.fadeInMs, offsetMs),
  };
  const tailLen = clipEndMs(clip) - at;
  const tail: VideoAudioClip = {
    ...clip,
    id: crypto.randomUUID(),
    sourceInMs: splitSourceMs,
    timelineStartMs: at,
    fadeInMs: 0,
    fadeOutMs: Math.min(clip.fadeOutMs, tailLen),
  };
  const clips = project.audioTrack.clips.flatMap((c) => (c.id === clipId ? [head, tail] : [c]));
  return commitProject({ ...project, audioTrack: { clips } });
}

export function duplicateAudioClip(project: VideoEditProject, clipId: string): VideoEditProject {
  const clip = audioClipById(project, clipId);
  if (!clip) return project;
  const durationMs = projectDurationMs(project);
  const startMs = clipEndMs(clip);
  if (startMs > durationMs - MIN_CLIP_MS) return project;
  const lengthMs = Math.min(clipLengthMs(clip), durationMs - startMs);
  const copy: VideoAudioClip = {
    ...clip,
    id: crypto.randomUUID(),
    timelineStartMs: startMs,
    sourceOutMs: clip.sourceInMs + lengthMs,
    fadeInMs: Math.min(clip.fadeInMs, Math.floor(lengthMs / 2)),
    fadeOutMs: Math.min(clip.fadeOutMs, Math.floor(lengthMs / 2)),
  };
  const blocked = project.audioTrack.clips.some(
    (candidate) =>
      candidate.lane === copy.lane &&
      candidate.timelineStartMs < clipEndMs(copy) &&
      copy.timelineStartMs < clipEndMs(candidate),
  );
  if (blocked) return project;
  return commitProject({ ...project, audioTrack: { clips: [...project.audioTrack.clips, copy] } });
}

export function removeAudioClip(project: VideoEditProject, clipId: string): VideoEditProject {
  return commitProject({
    ...project,
    audioTrack: { clips: project.audioTrack.clips.filter(({ id }) => id !== clipId) },
  });
}

function replace(project: VideoEditProject, edited: VideoAudioClip): VideoEditProject {
  return commitProject({
    ...project,
    audioTrack: {
      clips: project.audioTrack.clips.map((clip) => (clip.id === edited.id ? edited : clip)),
    },
  });
}
