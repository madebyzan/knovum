import { videoEditProjectSchema } from "../../model/nodes/schema/video-edit";
import { MIN_CLIP_MS, type VideoEditProject } from "../project";
import { clipEndMs, projectDurationMs } from "../timing";

/**
 * Normalizes a project after an edit and validates it. Edits produce the intended structure; this
 * boundary owns the shared consequences: ordering, media that fell outside the timeline, assets
 * that lost their last reference, and the structural contract itself.
 */
export function commitProject(project: VideoEditProject): VideoEditProject {
  const clips = project.videoTrack.clips.toSorted(
    (left, right) => left.timelineStartMs - right.timelineStartMs,
  );
  const transitions = project.transitions.filter((transition) => adjacentPair(clips, transition.fromClipId));
  const normalized = { ...project, videoTrack: { clips }, transitions };
  const duration = projectDurationMs(normalized);
  return videoEditProjectSchema.parse(
    pruneAssets({
      ...normalized,
      audioTrack: { clips: clampAudio(normalized.audioTrack.clips, duration) },
      textTrack: { items: clampText(normalized.textTrack.items, duration) },
    }),
  );
}

/** Music follows the video track: it is trimmed to the timeline, not discarded. */
function clampAudio(clips: VideoEditProject["audioTrack"]["clips"], durationMs: number) {
  const kept: VideoEditProject["audioTrack"]["clips"] = [];
  for (const clip of clips) {
    if (clip.timelineStartMs > durationMs - MIN_CLIP_MS) continue;
    const overflowMs = clipEndMs(clip) - durationMs;
    kept.push(overflowMs > 0 ? { ...clip, sourceOutMs: clip.sourceOutMs - overflowMs } : clip);
  }
  return kept;
}

function clampText(items: VideoEditProject["textTrack"]["items"], durationMs: number) {
  const kept: VideoEditProject["textTrack"]["items"] = [];
  for (const item of items) {
    const endMs = Math.min(item.endMs, durationMs);
    if (endMs - item.startMs >= 100) kept.push({ ...item, endMs });
  }
  return kept;
}

function adjacentPair(clips: VideoEditProject["videoTrack"]["clips"], clipId: string) {
  const index = clips.findIndex(({ id }) => id === clipId);
  return index >= 0 && index + 1 < clips.length;
}

/** The owning source stays referenced even when no clip uses it. */
export function pruneAssets(project: VideoEditProject): VideoEditProject {
  const references = new Set([
    project.assets[0]!.nodeId,
    ...project.videoTrack.clips.map(({ assetId }) => assetId),
    ...project.audioTrack.clips.map(({ assetId }) => assetId),
  ]);
  return { ...project, assets: project.assets.filter(({ nodeId }) => references.has(nodeId)) };
}

/**
 * Ripple: everything that starts at or after `atMs` moves by `deltaMs`, so inserting or removing
 * media retimes later clips, music and text in one operation.
 */
export function shiftAfter(project: VideoEditProject, atMs: number, deltaMs: number): VideoEditProject {
  if (deltaMs === 0) return project;
  const shift = (value: number) => (value >= atMs ? Math.max(0, value + deltaMs) : value);
  return {
    ...project,
    videoTrack: {
      clips: project.videoTrack.clips.map((clip) => ({
        ...clip,
        timelineStartMs: shift(clip.timelineStartMs),
      })),
    },
    audioTrack: {
      clips: project.audioTrack.clips.map((clip) => ({
        ...clip,
        timelineStartMs: shift(clip.timelineStartMs),
      })),
    },
    textTrack: {
      items: project.textTrack.items.map((item) =>
        item.startMs >= atMs
          ? { ...item, startMs: shift(item.startMs), endMs: Math.max(100, item.endMs + deltaMs) }
          : item,
      ),
    },
  };
}

/** Timeline positions a clip edge can snap to: clip bounds, text bounds and audio bounds. */
export function editPoints(project: VideoEditProject) {
  const points = new Set<number>([0, projectDurationMs(project)]);
  for (const clip of project.videoTrack.clips) {
    points.add(clip.timelineStartMs);
    points.add(clipEndMs(clip));
  }
  for (const clip of project.audioTrack.clips) {
    points.add(clip.timelineStartMs);
    points.add(clipEndMs(clip));
  }
  for (const item of project.textTrack.items) {
    points.add(item.startMs);
    points.add(item.endMs);
  }
  return [...points].toSorted((left, right) => left - right);
}

export function clipById(project: VideoEditProject, clipId: string) {
  return project.videoTrack.clips.find(({ id }) => id === clipId);
}

export function assetOf(project: VideoEditProject, assetId: string) {
  return project.assets.find(({ nodeId }) => nodeId === assetId);
}
