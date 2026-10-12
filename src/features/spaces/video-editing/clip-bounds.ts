import type { VideoClip, VideoEditProject } from "./project";

export function clipTrimBounds(project: VideoEditProject, clip: VideoClip) {
  const index = project.videoTrack.clips.findIndex(({ id }) => id === clip.id);
  const asset = project.assets.find(({ nodeId }) => nodeId === clip.assetId)!;
  const incoming = project.videoTrack.clips[index - 1]?.transition?.durationMs ?? 0;
  const minimum = Math.max(
    100 + incoming + (clip.transition?.durationMs ?? 0),
    clip.fadeInMs + clip.fadeOutMs,
  );
  return { minimum, maximum: asset.kind === "image" ? 86_400_000 : asset.durationMs };
}

export function stepTrimFrameMs(timeMs: number, frameRate: number, direction: -1 | 1) {
  const frame = (timeMs * frameRate) / 1000;
  const tolerance = frameRate / 2000 + 1e-7;
  const index = direction === 1 ? Math.floor(frame + tolerance) + 1 : Math.ceil(frame - tolerance) - 1;
  return Math.max(0, Math.round((index * 1000) / frameRate));
}
