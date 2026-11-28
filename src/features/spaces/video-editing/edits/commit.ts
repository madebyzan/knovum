import { videoEditProjectSchema } from "../../model/nodes/schema/video-edit";
import { MIN_CLIP_MS, type VideoAudioClip, type VideoEditProject } from "../project";
import { clipEndMs, clipLengthMs, projectDurationMs } from "../timing";

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
  const durationMs = projectDurationMs(normalized);
  return videoEditProjectSchema.parse(
    pruneAssets({
      ...normalized,
      audioTrack: { clips: normalizeAudio(normalized.audioTrack.clips, durationMs) },
      textTrack: { items: clampText(normalized.textTrack.items, durationMs) },
    }),
  );
}

/** Music follows the video track: sorted per lane and trimmed to the timeline without overlap. */
function normalizeAudio(clips: VideoEditProject["audioTrack"]["clips"], durationMs: number) {
  const sorted = clips.toSorted(
    (left, right) => left.lane - right.lane || left.timelineStartMs - right.timelineStartMs,
  );
  const kept: VideoAudioClip[] = [];
  const laneEnd = new Map<number, number>();
  for (const clip of sorted) {
    const earliestStart = Math.max(clip.timelineStartMs, laneEnd.get(clip.lane) ?? 0);
    if (earliestStart > durationMs - MIN_CLIP_MS) continue;
    const adjusted =
      earliestStart === clip.timelineStartMs ? clip : { ...clip, timelineStartMs: earliestStart };
    const overflowMs = clipEndMs(adjusted) - durationMs;
    const clamped =
      overflowMs > 0 ? { ...adjusted, sourceOutMs: adjusted.sourceOutMs - overflowMs } : adjusted;
    if (clipLengthMs(clamped) < MIN_CLIP_MS) continue;
    kept.push(clamped);
    laneEnd.set(clamped.lane, clipEndMs(clamped));
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
