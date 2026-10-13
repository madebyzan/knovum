import { trimAudioClip } from "../edits/audio";
import { trimClip, type TrimEdge } from "../edits/trim";
import type { VideoAudioClip, VideoClip, VideoEditProject } from "../project";
import { clipEndMs, frameDurationMs } from "../timing";

/** Keyboard trimming moves an edge by exactly one output frame. */
export function stepTrimFrame(project: VideoEditProject, clip: VideoClip, edge: TrimEdge, direction: -1 | 1) {
  const target = edgePosition(clip, edge) + direction * frameDurationMs(project);
  const next = trimClip(project, clip.id, edge, target);
  return next === project ? undefined : next;
}

export function stepAudioTrimFrame(
  project: VideoEditProject,
  clip: VideoAudioClip,
  edge: TrimEdge,
  direction: -1 | 1,
) {
  const target = edgePosition(clip, edge) + direction * frameDurationMs(project);
  const next = trimAudioClip(project, clip.id, edge, target);
  return next === project ? undefined : next;
}

function edgePosition(clip: VideoClip | VideoAudioClip, edge: TrimEdge) {
  return edge === "start" ? clip.timelineStartMs : clipEndMs(clip);
}
