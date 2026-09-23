import type { SpaceVideoPayload } from "../model/nodes/node-payload";

export type VideoEditProject = NonNullable<SpaceVideoPayload["videoEditProject"]>;
export type VideoClip = VideoEditProject["videoTrack"]["clips"][number];
export type VideoTextOverlay = VideoEditProject["textTrack"]["items"][number];

export function createVideoEditProject(sourceRevision: string, durationMs: number): VideoEditProject {
  return {
    version: 1,
    sourceRevision,
    videoTrack: {
      clips: [{ id: crypto.randomUUID(), startMs: 0, endMs: durationMs, volume: 1 }],
    },
    textTrack: { items: [] },
    transform: { rotation: 0, flipHorizontal: false, flipVertical: false },
  };
}

export function projectDuration(project: VideoEditProject) {
  return project.videoTrack.clips.reduce((duration, clip) => duration + clipDuration(clip), 0);
}

export function clipDuration(clip: VideoClip) {
  return clip.endMs - clip.startMs;
}

export function splitClip(project: VideoEditProject, timelineMs: number): VideoEditProject {
  const location = locateTimelineTime(project, timelineMs);
  if (!location || location.offsetMs < 100 || clipDuration(location.clip) - location.offsetMs < 100)
    return project;
  const splitMs = location.clip.startMs + location.offsetMs;
  const clips = project.videoTrack.clips.flatMap((clip) =>
    clip.id === location.clip.id
      ? [
          { ...clip, id: crypto.randomUUID(), endMs: splitMs },
          { ...clip, id: crypto.randomUUID(), startMs: splitMs },
        ]
      : [clip],
  );
  return { ...project, videoTrack: { clips } };
}

export function removeClip(project: VideoEditProject, clipId: string): VideoEditProject {
  if (project.videoTrack.clips.length === 1) return project;
  return {
    ...project,
    videoTrack: { clips: project.videoTrack.clips.filter(({ id }) => id !== clipId) },
  };
}

export function locateTimelineTime(project: VideoEditProject, timelineMs: number) {
  let cursor = 0;
  for (const [clipIndex, clip] of project.videoTrack.clips.entries()) {
    const duration = clipDuration(clip);
    if (timelineMs < cursor + duration)
      return { clip, clipIndex, offsetMs: timelineMs - cursor, timelineStartMs: cursor };
    cursor += duration;
  }
  return undefined;
}

export function clipTimelineStart(project: VideoEditProject, clipId: string) {
  let cursor = 0;
  for (const clip of project.videoTrack.clips) {
    if (clip.id === clipId) return cursor;
    cursor += clipDuration(clip);
  }
  return 0;
}

export function addTextOverlay(project: VideoEditProject, startMs: number): VideoEditProject {
  const duration = projectDuration(project);
  const endMs = Math.min(duration, startMs + 3_000);
  const item: VideoTextOverlay = {
    id: crypto.randomUUID(),
    startMs: Math.min(startMs, Math.max(0, duration - 500)),
    endMs: Math.max(Math.min(endMs, duration), Math.min(startMs, duration - 500) + 500),
    text: "Text",
    placement: "bottom",
  };
  return { ...project, textTrack: { items: [...project.textTrack.items, item] } };
}
