import type { VideoAsset, VideoEditProject } from "../project";
import { createVideoClip, STILL_IMAGE_MS } from "../project";
import { clipEndMs, clipLengthMs, compositionAt, projectDurationMs } from "../timing";
import { commitProject, shiftAfter } from "./commit";

const AUDIO_LANES = 4;

/**
 * Adds a material to the timeline at a position. Visual media is inserted at the nearest clip
 * boundary and rolls the rest of the timeline forward; audio takes the first free lane.
 */
export function insertMaterial(project: VideoEditProject, asset: VideoAsset, atMs: number): VideoEditProject {
  const existing = project.assets.find(({ nodeId }) => nodeId === asset.nodeId);
  if (existing && existing.sourceRevision !== asset.sourceRevision)
    throw new Error("Video material changed; reopen the project with its current source");
  const assets = existing ? project.assets : [...project.assets, asset];
  const at = Math.max(0, Math.round(atMs));
  return asset.kind === "audio"
    ? insertAudio({ ...project, assets }, asset, at)
    : insertVisual({ ...project, assets }, asset, at);
}

function insertVisual(project: VideoEditProject, asset: VideoAsset, atMs: number) {
  const inside = compositionAt(project, atMs).layers[0];
  const insertAt = inside ? inside.clip.timelineStartMs : atMs;
  const clip = createVideoClip(asset.nodeId, {
    sourceOutMs: asset.kind === "image" ? STILL_IMAGE_MS : asset.durationMs,
    timelineStartMs: insertAt,
  });
  const rolled = shiftAfter(project, insertAt, clipLengthMs(clip));
  return commitProject({ ...rolled, videoTrack: { clips: [...rolled.videoTrack.clips, clip] } });
}

function insertAudio(project: VideoEditProject, asset: VideoAsset, atMs: number) {
  const durationMs = projectDurationMs(project);
  const start = Math.min(atMs, Math.max(0, durationMs - 100));
  const length = Math.min(asset.durationMs, durationMs - start);
  if (length < 100) return project;
  const lane = freeLane(project, start, start + length);
  if (lane === undefined) return project;
  return commitProject({
    ...project,
    audioTrack: {
      clips: [
        ...project.audioTrack.clips,
        {
          id: crypto.randomUUID(),
          assetId: asset.nodeId,
          sourceInMs: 0,
          sourceOutMs: length,
          timelineStartMs: start,
          lane,
          volume: 1,
          fadeInMs: 0,
          fadeOutMs: 0,
        },
      ],
    },
  });
}

/** First lane whose clips do not cover the requested span. */
export function freeLane(project: VideoEditProject, startMs: number, endMs: number, ignoreClipId?: string) {
  for (let lane = 0; lane < AUDIO_LANES; lane += 1) {
    const occupied = project.audioTrack.clips.some(
      (clip) =>
        clip.lane === lane &&
        clip.id !== ignoreClipId &&
        clip.timelineStartMs < endMs &&
        clipEndMs(clip) > startMs,
    );
    if (!occupied) return lane;
  }
  return undefined;
}
