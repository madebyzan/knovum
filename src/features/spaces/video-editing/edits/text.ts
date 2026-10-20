import { clamp } from "es-toolkit";

import type { VideoEditProject, VideoTextOverlay, VideoTextStyle } from "../project";
import { createTextOverlay, MIN_CLIP_MS } from "../project";
import { projectDurationMs } from "../timing";
import { commitProject } from "./commit";

const DEFAULT_TEXT_MS = 3_000;

export function addTextOverlay(
  project: VideoEditProject,
  atMs: number,
  style: VideoTextStyle,
  text: string,
): VideoEditProject {
  const durationMs = projectDurationMs(project);
  const startMs = clamp(Math.round(atMs), 0, Math.max(0, durationMs - MIN_CLIP_MS));
  const endMs = clamp(startMs + DEFAULT_TEXT_MS, startMs + MIN_CLIP_MS, durationMs);
  return commitProject({
    ...project,
    textTrack: { items: [...project.textTrack.items, createTextOverlay(startMs, endMs, style, text)] },
  });
}

export function updateTextOverlay(
  project: VideoEditProject,
  id: string,
  patch: Partial<VideoTextOverlay>,
): VideoEditProject {
  const item = project.textTrack.items.find((candidate) => candidate.id === id);
  if (!item) return project;
  const durationMs = projectDurationMs(project);
  const merged = { ...item, ...patch };
  const next = {
    ...merged,
    startMs: clamp(merged.startMs, 0, Math.max(0, durationMs - MIN_CLIP_MS)),
    endMs: clamp(merged.endMs, merged.startMs + MIN_CLIP_MS, durationMs),
  };
  if (!next.text.trim()) return project;
  return commitProject({
    ...project,
    textTrack: {
      items: project.textTrack.items.map((candidate) => (candidate.id === id ? next : candidate)),
    },
  });
}

export function updateTextStyle(
  project: VideoEditProject,
  id: string,
  patch: Partial<VideoTextStyle>,
): VideoEditProject {
  const item = project.textTrack.items.find((candidate) => candidate.id === id);
  return item ? updateTextOverlay(project, id, { style: { ...item.style, ...patch } }) : project;
}

/** Moves a text block in time while keeping its length. */
export function moveTextOverlay(project: VideoEditProject, id: string, startMs: number): VideoEditProject {
  const item = project.textTrack.items.find((candidate) => candidate.id === id);
  if (!item) return project;
  const length = item.endMs - item.startMs;
  const start = clamp(Math.round(startMs), 0, Math.max(0, projectDurationMs(project) - length));
  return updateTextOverlay(project, id, { startMs: start, endMs: start + length });
}

export function removeTextOverlay(project: VideoEditProject, id: string): VideoEditProject {
  return commitProject({
    ...project,
    textTrack: { items: project.textTrack.items.filter((item) => item.id !== id) },
  });
}
