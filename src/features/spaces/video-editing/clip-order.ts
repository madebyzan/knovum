import { videoEditProjectSchema } from "../model/nodes/schema/video-edit";
import type { VideoClip, VideoEditProject } from "./project";
import { spliceTimeline } from "./timeline-edits";
import { clipDurationMs, clipTimelineStartMs, projectDurationMs } from "./timing";

export function duplicateClip(project: VideoEditProject, clipId: string): VideoEditProject {
  const index = project.videoTrack.clips.findIndex(({ id }) => id === clipId);
  const original = project.videoTrack.clips[index];
  if (!original) return project;
  const clips = [...project.videoTrack.clips];
  clips.splice(index + 1, 0, { ...original, id: crypto.randomUUID(), transition: null });
  const next = { ...project, videoTrack: { clips: preserveBoundaries(project, clips) } };
  return videoEditProjectSchema.parse(
    spliceTimeline(next, [
      {
        atMs:
          clipTimelineStartMs(project, clipId) +
          clipDurationMs(original) -
          (original.transition?.durationMs ?? 0),
        removedMs: 0,
        insertedMs: projectDurationMs(next) - projectDurationMs(project),
      },
    ]),
  );
}
export function moveClip(project: VideoEditProject, clipId: string, direction: -1 | 1): VideoEditProject {
  const index = project.videoTrack.clips.findIndex(({ id }) => id === clipId);
  const target = index + direction;
  if (index < 0 || target < 0 || target >= project.videoTrack.clips.length) return project;
  return moveClipTo(project, clipId, target);
}
export function moveClipTo(project: VideoEditProject, clipId: string, target: number): VideoEditProject {
  const clips = [...project.videoTrack.clips];
  const index = clips.findIndex(({ id }) => id === clipId);
  if (index < 0 || target < 0 || target >= clips.length || index === target) return project;
  const [clip] = clips.splice(index, 1);
  clips.splice(target, 0, clip!);
  return videoEditProjectSchema.parse({
    ...project,
    videoTrack: { clips: preserveBoundaries(project, clips) },
  });
}
export function preserveBoundaries(project: VideoEditProject, clips: VideoClip[]) {
  const neighbors = new Map(
    project.videoTrack.clips.map((clip, index) => [clip.id, project.videoTrack.clips[index + 1]?.id]),
  );
  return clips.map((clip, index) =>
    clip.transition && neighbors.get(clip.id) !== clips[index + 1]?.id ? { ...clip, transition: null } : clip,
  );
}
