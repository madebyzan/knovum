import type { VideoAsset, VideoClip, VideoEditProject } from "./project";
import { fadeGain, locateTimelineFrames, projectDurationMs } from "./timing";

export function previewTimeMs(project: VideoEditProject, timelineMs: number) {
  const lastFrame =
    ((Math.ceil((projectDurationMs(project) * project.output.frameRate) / 1000) - 1) * 1000) /
    project.output.frameRate;
  return Math.min(timelineMs, Math.max(0, lastFrame));
}

export function previewFrames(project: VideoEditProject, timelineMs: number) {
  const frames = locateTimelineFrames(project, previewTimeMs(project, timelineMs));
  const progress = frames[1]?.progress ?? 1;
  const kind = frames[0]?.clip.transition?.kind;
  const overlapping = frames.length === 2;
  const dissolve = overlapping && kind === "dissolve";
  return frames.map((frame, index) => {
    const weight = overlapping ? (index === 0 ? 1 - progress : progress) : 1;
    const fade = fadeGain(frame.clip, frame.offsetMs);
    return Object.assign({}, frame, {
      opacity: dissolve ? weight : 1,
      dissolve,
      clipPath: overlapping && index === 1 ? transitionClipPath(kind, progress) : undefined,
      fade,
      gain: frame.clip.volume * weight * fade,
    });
  });
}
function transitionClipPath(kind: string | undefined, progress: number) {
  if (kind === "wipeLeft") return `inset(0 0 0 ${(1 - progress) * 100}%)`;
  if (kind === "wipeRight") return `inset(0 ${(1 - progress) * 100}% 0 0)`;
  return undefined;
}

export function previewGeometry(
  asset: VideoAsset,
  transform: VideoClip["transform"],
  output: VideoEditProject["output"],
) {
  const quarterTurn = transform.rotation === 90 || transform.rotation === 270;
  const width = quarterTurn ? asset.height : asset.width;
  const height = quarterTurn ? asset.width : asset.height;
  const ratio =
    transform.fit === "contain"
      ? Math.min(output.width / width, output.height / height)
      : Math.max(output.width / width, output.height / height);
  return { width: asset.width * ratio * transform.zoom, height: asset.height * ratio * transform.zoom };
}
