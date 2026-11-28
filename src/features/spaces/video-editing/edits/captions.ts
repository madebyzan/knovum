import { clamp, sortBy } from "es-toolkit";

import type { SubtitleCue } from "../../media/subtitle-codec";
import type { VideoMaterialNode } from "../material";
import {
  createTextOverlay,
  defaultTextStyle,
  MIN_CLIP_MS,
  type VideoEditProject,
  type VideoTextOverlay,
  type VideoTextStyle,
} from "../project";
import { clipLengthMs, projectDurationMs } from "../timing";
import { commitProject } from "./commit";

const CAPTION_Y = 0.88;
const CJK_CHAR_RE = /[\p{Script=Han}\p{Script=Hiragana}\p{Script=Katakana}\p{Script=Hangul}]/u;
const BREAK_CHARS = new Set(["。", "！", "？", "；", "，", "、", ".", "!", "?", ";", ",", " "]);

export function defaultCaptionStyle(family: string): VideoTextStyle {
  return { ...defaultTextStyle(family), size: 36, weight: 600, decorationStrength: 2, align: "center" };
}

function charBudget(text: string): number {
  let cjkCount = 0;
  for (const ch of text) {
    if (CJK_CHAR_RE.test(ch)) cjkCount += 1;
  }
  return cjkCount * 3 >= text.length ? 20 : 44;
}

/** Splits long transcript text into balanced broadcast-readable caption lines. */
export function splitCaptionText(raw: string): string[] {
  const clean = raw.replace(/\s+/gu, " ").trim();
  if (!clean) return [];
  const budget = charBudget(clean);
  const chars = [...clean];
  if (chars.length <= budget) return [clean];
  const chunks: string[] = [];
  let cursor = 0;
  while (cursor < chars.length) {
    const remaining = chars.length - cursor;
    if (remaining <= budget) {
      chunks.push(chars.slice(cursor).join("").trim());
      break;
    }
    const minCut = cursor + Math.floor(budget * 0.5);
    const maxCut = Math.min(chars.length, cursor + budget);
    let cut = maxCut;
    for (let i = maxCut; i >= minCut; i--) {
      if (BREAK_CHARS.has(chars[i - 1]!)) {
        cut = i;
        break;
      }
    }
    const piece = chars.slice(cursor, cut).join("").trim();
    if (piece) chunks.push(piece);
    cursor = cut;
    while (cursor < chars.length && chars[cursor] === " ") cursor += 1;
  }
  return chunks.filter(Boolean);
}

function projectClipCaptions(
  clip: VideoEditProject["videoTrack"]["clips"][number],
  material: VideoMaterialNode | undefined,
  style: VideoTextStyle,
  durationMs: number,
): VideoTextOverlay[] {
  if (!material || material.data.kind !== "video") return [];
  const knowledge = material.data.mediaKnowledge;
  if (!knowledge || knowledge.sourceRevision !== material.data.sourceRevision) return [];
  const clipOutMs = clip.sourceInMs + clipLengthMs(clip);
  const shiftMs = clip.timelineStartMs - clip.sourceInMs;
  const overlays: VideoTextOverlay[] = [];
  for (const segment of knowledge.transcript.segments) {
    const srcStart = Math.max(segment.startMs, clip.sourceInMs);
    const srcEnd = Math.min(segment.endMs, clipOutMs);
    if (srcEnd - srcStart < MIN_CLIP_MS) continue;
    const pieces = splitCaptionText(segment.text);
    const totalChars = pieces.reduce((sum, piece) => sum + [...piece].length, 0) || 1;
    let pieceStart = srcStart;
    for (let i = 0; i < pieces.length; i++) {
      const piece = pieces[i]!;
      const share = (srcEnd - srcStart) * ([...piece].length / totalChars);
      const pieceEnd = i === pieces.length - 1 ? srcEnd : Math.round(pieceStart + share);
      const tlStart = clamp(Math.round(pieceStart + shiftMs), 0, Math.max(0, durationMs - MIN_CLIP_MS));
      const tlEnd = clamp(Math.round(pieceEnd + shiftMs), tlStart + MIN_CLIP_MS, durationMs);
      overlays.push({ ...createTextOverlay(tlStart, tlEnd, { ...style }, piece), x: 0.5, y: CAPTION_Y });
      pieceStart = pieceEnd;
    }
  }
  return overlays;
}

/** Projects transcribed speech from every timeline video clip into frame-accurate captions. */
export function generateCaptionsFromTranscripts(
  project: VideoEditProject,
  materials: ReadonlyMap<string, VideoMaterialNode>,
  style: VideoTextStyle,
): { project: VideoEditProject; count: number } {
  const durationMs = projectDurationMs(project);
  const generated = project.videoTrack.clips.flatMap((clip) =>
    projectClipCaptions(clip, materials.get(clip.assetId), style, durationMs),
  );
  if (!generated.length) return { project, count: 0 };
  return {
    project: commitProject({ ...project, textTrack: { items: generated } }),
    count: generated.length,
  };
}

/** Replaces the caption track with cues parsed from an imported SRT or WebVTT file. */
export function importSubtitlesIntoProject(
  project: VideoEditProject,
  cues: readonly SubtitleCue[],
  style: VideoTextStyle,
): { project: VideoEditProject; count: number } {
  const durationMs = projectDurationMs(project);
  const items: VideoTextOverlay[] = [];
  for (const cue of cues) {
    if (cue.startMs > durationMs - MIN_CLIP_MS) continue;
    const startMs = clamp(Math.round(cue.startMs), 0, Math.max(0, durationMs - MIN_CLIP_MS));
    const endMs = clamp(Math.round(cue.endMs), startMs + MIN_CLIP_MS, durationMs);
    items.push({ ...createTextOverlay(startMs, endMs, { ...style }, cue.text), x: 0.5, y: CAPTION_Y });
  }
  if (!items.length) return { project, count: 0 };
  return { project: commitProject({ ...project, textTrack: { items } }), count: items.length };
}

/** Applies the selected caption's typography, decoration, and vertical placement to all captions. */
export function applyTextStyleToAll(project: VideoEditProject, sourceId: string): VideoEditProject {
  const source = project.textTrack.items.find((item) => item.id === sourceId);
  if (!source || project.textTrack.items.length < 2) return project;
  const items = project.textTrack.items.map((item) => ({
    ...item,
    x: source.x,
    y: source.y,
    style: { ...source.style },
  }));
  return commitProject({ ...project, textTrack: { items } });
}

export function canSplitTextAt(item: VideoTextOverlay, atMs: number): boolean {
  const at = Math.round(atMs);
  return at >= item.startMs + MIN_CLIP_MS && at <= item.endMs - MIN_CLIP_MS;
}

/** Splits a caption at the playhead, dividing text proportionally near a word/punctuation break. */
export function splitTextOverlayAt(project: VideoEditProject, id: string, atMs: number): VideoEditProject {
  const item = project.textTrack.items.find((candidate) => candidate.id === id);
  const at = Math.round(atMs);
  if (!item || !canSplitTextAt(item, at)) return project;
  const ratio = (at - item.startMs) / (item.endMs - item.startMs);
  const chars = [...item.text.trim()];
  const ideal = Math.max(1, Math.min(chars.length - 1, Math.round(chars.length * ratio)));
  let cut = ideal;
  for (let delta = 0; delta <= 8; delta++) {
    if (ideal + delta < chars.length && BREAK_CHARS.has(chars[ideal + delta - 1]!)) {
      cut = ideal + delta;
      break;
    }
    if (ideal - delta > 0 && BREAK_CHARS.has(chars[ideal - delta - 1]!)) {
      cut = ideal - delta;
      break;
    }
  }
  const firstText = chars.slice(0, cut).join("").trim() || item.text;
  const secondText = chars.slice(cut).join("").trim() || item.text;
  const head: VideoTextOverlay = { ...item, endMs: at, text: firstText };
  const tail: VideoTextOverlay = {
    ...item,
    id: crypto.randomUUID(),
    startMs: at,
    text: secondText,
    style: { ...item.style },
  };
  const items = project.textTrack.items.flatMap((c) => (c.id === id ? [head, tail] : [c]));
  return commitProject({ ...project, textTrack: { items } });
}

/** Merges the selected caption with the chronologically next caption on the track. */
export function mergeTextOverlayWithNext(project: VideoEditProject, id: string): VideoEditProject {
  const ordered = sortBy(project.textTrack.items, [(item) => item.startMs, (item) => item.endMs]);
  const index = ordered.findIndex((item) => item.id === id);
  if (index < 0 || index >= ordered.length - 1) return project;
  const current = ordered[index]!;
  const next = ordered[index + 1]!;
  const merged: VideoTextOverlay = {
    ...current,
    endMs: Math.max(current.endMs, next.endMs),
    text: `${current.text.trim()}\n${next.text.trim()}`,
  };
  const items = project.textTrack.items
    .filter((item) => item.id !== next.id)
    .map((item) => (item.id === current.id ? merged : item));
  return commitProject({ ...project, textTrack: { items } });
}
