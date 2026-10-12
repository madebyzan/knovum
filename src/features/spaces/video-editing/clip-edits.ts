import { videoEditProjectSchema } from "../model/nodes/schema/video-edit";
import { type VideoClip, type VideoEditProject, type VideoTransition } from "./project";
import { spliceTimeline } from "./timeline-edits";
import {
  canSplitClip,
  clipDurationMs,
  clipTimelineStartMs,
  locateTimelineTime,
  projectDurationMs,
} from "./timing";

export function splitClip(project: VideoEditProject, timelineMs: number): VideoEditProject {
  timelineMs = Math.round(timelineMs);
  if (!canSplitClip(project, timelineMs)) return project;
  const location = locateTimelineTime(project, timelineMs)!;
  const splitMs = location.clip.startMs + location.offsetMs;
  const clips = project.videoTrack.clips.flatMap((clip) =>
    clip.id === location.clip.id
      ? [
          { ...clip, id: crypto.randomUUID(), endMs: splitMs, fadeOutMs: 0, transition: null },
          { ...clip, id: crypto.randomUUID(), startMs: splitMs, fadeInMs: 0 },
        ]
      : [clip],
  );
  return videoEditProjectSchema.parse({ ...project, videoTrack: { clips } });
}
export function removeClip(project: VideoEditProject, clipId: string): VideoEditProject {
  if (project.videoTrack.clips.length === 1) return project;
  const index = project.videoTrack.clips.findIndex(({ id }) => id === clipId);
  const clip = project.videoTrack.clips[index];
  if (!clip) return project;
  const clips = project.videoTrack.clips
    .filter(({ id }) => id !== clipId)
    .map((entry, position) =>
      position === index - 1 ? Object.assign({}, entry, { transition: null }) : entry,
    );
  const next = { ...project, videoTrack: { clips } };
  const removedMs = projectDurationMs(project) - projectDurationMs(next);
  return videoEditProjectSchema.parse(
    spliceTimeline(next, [{ atMs: clipTimelineStartMs(project, clipId), removedMs, insertedMs: 0 }]),
  );
}
export function replaceClip(project: VideoEditProject, edited: VideoClip): VideoEditProject {
  const original = project.videoTrack.clips.find(({ id }) => id === edited.id);
  if (!original) return project;
  const next = {
    ...project,
    videoTrack: { clips: project.videoTrack.clips.map((clip) => (clip.id === edited.id ? edited : clip)) },
  };
  const start = clipTimelineStartMs(project, edited.id);
  const startDelta = original.startMs - edited.startMs;
  const endDelta = edited.endMs - original.endMs;
  return videoEditProjectSchema.parse(
    spliceTimeline(next, [
      { atMs: start, removedMs: Math.max(0, -startDelta), insertedMs: Math.max(0, startDelta) },
      {
        atMs: start + clipDurationMs(original) + startDelta + Math.min(0, endDelta),
        removedMs: Math.max(0, -endDelta),
        insertedMs: Math.max(0, endDelta),
      },
    ]),
  );
}
export function setTransition(
  project: VideoEditProject,
  clipId: string,
  transition: VideoTransition | null,
): VideoEditProject {
  const start = clipTimelineStartMs(project, clipId);
  const original = project.videoTrack.clips.find(({ id }) => id === clipId)!;
  const delta = (transition?.durationMs ?? 0) - (original.transition?.durationMs ?? 0);
  const next = {
    ...project,
    videoTrack: {
      clips: project.videoTrack.clips.map((clip) =>
        clip.id === clipId ? Object.assign({}, clip, { transition }) : clip,
      ),
    },
  };
  return videoEditProjectSchema.parse(
    spliceTimeline(next, [
      {
        atMs:
          start +
          clipDurationMs(original) -
          Math.max(transition?.durationMs ?? 0, original.transition?.durationMs ?? 0),
        removedMs: Math.max(0, delta),
        insertedMs: Math.max(0, -delta),
      },
    ]),
  );
}
