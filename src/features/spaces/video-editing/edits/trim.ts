import type { VideoClip, VideoEditProject } from "../project";
import { MIN_CLIP_MS } from "../project";
import { clipEndMs, clipHeadroom, clipIndexOf, clipLengthMs } from "../timing";
import { assetOf, clipById, commitProject } from "./commit";

export type TrimEdge = "start" | "end";

/** Range the dragged edge may occupy, in timeline milliseconds. */
export interface TrimBounds {
  minMs: number;
  maxMs: number;
}

function minimumLength(project: VideoEditProject, clip: VideoClip) {
  const { incomingMs, outgoingMs } = clipHeadroom(project, clip);
  return Math.max(MIN_CLIP_MS + incomingMs + outgoingMs, clip.fadeInMs + clip.fadeOutMs);
}

function maximumLength(project: VideoEditProject, clip: VideoClip) {
  const asset = assetOf(project, clip.assetId);
  if (!asset || asset.kind === "image") return 86_400_000;
  return asset.durationMs - clip.sourceInMs;
}

function trimBounds(project: VideoEditProject, clipId: string, edge: TrimEdge): TrimBounds | undefined {
  const index = clipIndexOf(project, clipId);
  const clip = project.videoTrack.clips[index];
  if (!clip) return undefined;
  const { incomingMs, outgoingMs } = clipHeadroom(project, clip);
  const minLength = minimumLength(project, clip);
  if (edge === "start") {
    if (incomingMs > 0) return undefined;
    const previous = index > 0 ? project.videoTrack.clips[index - 1] : undefined;
    return {
      minMs: Math.max(previous ? clipEndMs(previous) : 0, clip.timelineStartMs - clip.sourceInMs),
      maxMs: clip.timelineStartMs + clipLengthMs(clip) - minLength,
    };
  }
  if (outgoingMs > 0) return undefined;
  const next = project.videoTrack.clips[index + 1];
  return {
    minMs: clip.timelineStartMs + minLength,
    maxMs: Math.min(
      next ? next.timelineStartMs : Number.MAX_SAFE_INTEGER,
      clip.timelineStartMs + maximumLength(project, clip),
    ),
  };
}

/** Moves one clip edge to a timeline position. The opposite edge and neighbours stay put. */
export function trimClip(
  project: VideoEditProject,
  clipId: string,
  edge: TrimEdge,
  timelineMs: number,
): VideoEditProject {
  const bounds = trimBounds(project, clipId, edge);
  const clip = clipById(project, clipId);
  if (!bounds || !clip) return project;
  const position = Math.round(Math.min(Math.max(timelineMs, bounds.minMs), bounds.maxMs));
  if (position === (edge === "start" ? clip.timelineStartMs : clipEndMs(clip))) return project;
  const edited =
    edge === "start"
      ? {
          ...clip,
          sourceInMs: clip.sourceInMs + (position - clip.timelineStartMs),
          timelineStartMs: position,
        }
      : { ...clip, sourceOutMs: clip.sourceOutMs + (position - clipEndMs(clip)) };
  return replace(project, edited);
}

/** Numeric source-window editing from the inspector; the timeline position is preserved. */
export function setClipWindow(
  project: VideoEditProject,
  clipId: string,
  window: { sourceInMs?: number; sourceOutMs?: number },
): VideoEditProject {
  const clip = clipById(project, clipId);
  if (!clip) return project;
  const next = { ...clip, ...window };
  const maxLength = maximumLength(project, next);
  if (
    next.sourceInMs < 0 ||
    next.sourceOutMs - next.sourceInMs < minimumLength(project, clip) ||
    next.sourceOutMs - next.sourceInMs > maxLength
  )
    return project;
  const trailing = trimBounds(project, clipId, "end");
  if (trailing && next.timelineStartMs + (next.sourceOutMs - next.sourceInMs) > trailing.maxMs)
    return project;
  return replace(project, next);
}

export function setClipFades(
  project: VideoEditProject,
  clipId: string,
  fades: { fadeInMs?: number; fadeOutMs?: number },
): VideoEditProject {
  const clip = clipById(project, clipId);
  if (!clip) return project;
  const next = { ...clip, ...fades };
  if (next.fadeInMs < 0 || next.fadeOutMs < 0 || next.fadeInMs + next.fadeOutMs > clipLengthMs(next))
    return project;
  return replace(project, next);
}

export function setClipTransform(
  project: VideoEditProject,
  clipId: string,
  transform: Partial<VideoClip["transform"]>,
): VideoEditProject {
  const clip = clipById(project, clipId);
  if (!clip) return project;
  return replace(project, { ...clip, transform: { ...clip.transform, ...transform } });
}

export function setClipVolume(project: VideoEditProject, clipId: string, volume: number): VideoEditProject {
  const clip = clipById(project, clipId);
  return clip ? replace(project, { ...clip, volume }) : project;
}

function replace(project: VideoEditProject, edited: VideoClip): VideoEditProject {
  return commitProject({
    ...project,
    videoTrack: {
      clips: project.videoTrack.clips.map((clip) => (clip.id === edited.id ? edited : clip)),
    },
  });
}
