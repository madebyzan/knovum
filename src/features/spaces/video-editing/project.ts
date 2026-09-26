import { sumBy } from "es-toolkit";

import type { SpaceVideoPayload } from "../model/nodes/node-payload";

export type VideoEditProject = NonNullable<SpaceVideoPayload["videoEditProject"]>;
export type VideoClip = VideoEditProject["videoTrack"]["clips"][number];
export type VideoTextOverlay = VideoEditProject["textTrack"]["items"][number];
export type VideoTextStyle = VideoTextOverlay["style"];

export function defaultVideoTextStyle(family: string): VideoTextStyle {
  return {
    family,
    weight: 400,
    size: 30,
    color: "#FFFFFF",
    preset: "outline",
  };
}

export function createVideoEditProject(
  sourceRevision: string,
  durationMs: number,
  sourceWidth: number,
  sourceHeight: number,
): VideoEditProject {
  return {
    version: 1,
    sourceRevision,
    sourceWidth,
    sourceHeight,
    videoTrack: {
      clips: [{ id: crypto.randomUUID(), startMs: 0, endMs: durationMs, volume: 1 }],
    },
    textTrack: { items: [] },
    transform: { rotation: 0, flipHorizontal: false, flipVertical: false },
  };
}

export function projectDuration(project: VideoEditProject) {
  return sumBy(project.videoTrack.clips, clipDuration);
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
  const clip = project.videoTrack.clips.find(({ id }) => id === clipId);
  if (!clip) return project;
  const timelineStart = clipTimelineStart(project, clipId);
  return fitTextToDuration({
    ...project,
    videoTrack: { clips: project.videoTrack.clips.filter(({ id }) => id !== clipId) },
    textTrack: { items: spliceText(project.textTrack.items, timelineStart, clipDuration(clip), 0) },
  });
}

export function replaceClip(project: VideoEditProject, edited: VideoClip): VideoEditProject {
  const original = project.videoTrack.clips.find(({ id }) => id === edited.id);
  if (!original) return project;
  const timelineStart = clipTimelineStart(project, edited.id);
  let text = project.textTrack.items;
  if (edited.startMs > original.startMs)
    text = spliceText(text, timelineStart, edited.startMs - original.startMs, 0);
  else if (edited.startMs < original.startMs)
    text = spliceText(text, timelineStart, 0, original.startMs - edited.startMs);
  if (edited.endMs < original.endMs)
    text = spliceText(text, timelineStart + clipDuration(edited), original.endMs - edited.endMs, 0);
  else if (edited.endMs > original.endMs)
    text = spliceText(
      text,
      timelineStart + original.endMs - edited.startMs,
      0,
      edited.endMs - original.endMs,
    );
  return fitTextToDuration({
    ...project,
    videoTrack: {
      clips: project.videoTrack.clips.map((clip) => (clip.id === edited.id ? edited : clip)),
    },
    textTrack: { items: text },
  });
}

function spliceText(items: VideoTextOverlay[], atMs: number, removedMs: number, insertedMs: number) {
  const mapTime = (value: number) => {
    if (value < atMs) return value;
    if (removedMs > 0 && value <= atMs + removedMs) return atMs;
    return value - removedMs + insertedMs;
  };
  return items
    .map((item) => ({ ...item, startMs: mapTime(item.startMs), endMs: mapTime(item.endMs) }))
    .filter((item) => item.endMs > item.startMs);
}

function fitTextToDuration(project: VideoEditProject): VideoEditProject {
  const duration = projectDuration(project);
  return {
    ...project,
    textTrack: {
      items: project.textTrack.items
        .filter((item) => item.startMs < duration)
        .map((item) => Object.assign({}, item, { endMs: Math.min(item.endMs, duration) })),
    },
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

export function addTextOverlay(project: VideoEditProject, startMs: number, family: string): VideoEditProject {
  const duration = projectDuration(project);
  const boundedStart = Math.max(0, Math.min(Math.round(startMs), Math.max(0, duration - 100)));
  const item: VideoTextOverlay = {
    id: crypto.randomUUID(),
    startMs: boundedStart,
    endMs: Math.min(duration, Math.max(boundedStart + 100, boundedStart + 3_000)),
    text: "Text",
    placement: "bottom",
    style: defaultVideoTextStyle(family),
  };
  return { ...project, textTrack: { items: [...project.textTrack.items, item] } };
}
