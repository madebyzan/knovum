import { videoEditProjectSchema } from "../model/nodes/schema/video-edit";
import { preserveBoundaries } from "./clip-order";
import { createVideoClip, type VideoAsset, type VideoEditProject } from "./project";
import { spliceTimeline } from "./timeline-edits";
import { locateTimelineTime, projectDurationMs } from "./timing";

export function appendAsset(
  project: VideoEditProject,
  asset: VideoAsset,
  atMs = projectDurationMs(project),
): VideoEditProject {
  const existing = project.assets.find(({ nodeId }) => nodeId === asset.nodeId);
  if (existing && existing.sourceRevision !== asset.sourceRevision)
    throw new Error("Video material changed; reopen the project with its current source");
  const assets = existing ? project.assets : [...project.assets, asset];
  if (asset.kind === "audio") {
    const timelineStartMs = Math.min(Math.round(atMs), Math.max(0, projectDurationMs(project) - 100));
    const endMs = Math.min(asset.durationMs, projectDurationMs(project) - timelineStartMs);
    return videoEditProjectSchema.parse({
      ...project,
      assets,
      audioTrack: {
        clips: [
          ...project.audioTrack.clips,
          {
            id: crypto.randomUUID(),
            assetId: asset.nodeId,
            startMs: 0,
            endMs,
            timelineStartMs,
            volume: 1,
            fadeInMs: 0,
            fadeOutMs: 0,
          },
        ],
      },
    });
  }
  const clips = [...project.videoTrack.clips];
  const location = locateTimelineTime(project, atMs);
  const index = location ? location.clipIndex : clips.length;
  clips.splice(index, 0, createVideoClip(asset.nodeId, asset.kind === "image" ? 3_000 : asset.durationMs));
  return videoEditProjectSchema.parse(
    spliceTimeline({ ...project, assets, videoTrack: { clips: preserveBoundaries(project, clips) } }, [
      {
        atMs: location?.timelineStartMs ?? projectDurationMs(project),
        removedMs: 0,
        insertedMs:
          projectDurationMs({ ...project, videoTrack: { clips: preserveBoundaries(project, clips) } }) -
          projectDurationMs(project),
      },
    ]),
  );
}
