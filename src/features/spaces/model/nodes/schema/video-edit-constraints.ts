import type { z } from "zod";

import type {
  videoAssetSchema,
  videoAudioClipSchema,
  videoClipSchema,
  videoTransitionSchema,
} from "./video-edit-media";

type Asset = z.infer<typeof videoAssetSchema>;
type Clip = z.infer<typeof videoClipSchema>;
type AudioClip = z.infer<typeof videoAudioClipSchema>;
type Transition = z.infer<typeof videoTransitionSchema>;
type TextItem = { id: string; startMs: number; endMs: number };
type Issue = (message: string) => void;

interface Composition {
  sourceRevision: string;
  assets: Asset[];
  videoTrack: { clips: Clip[] };
  transitions: Transition[];
  audioTrack: { clips: AudioClip[] };
  textTrack: { items: TextItem[] };
}

const MIN_CLIP_MS = 100;
const AUDIO_LANES = 4;

const length = (clip: Clip | AudioClip) => clip.sourceOutMs - clip.sourceInMs;
const timelineEnd = (clip: Clip | AudioClip) => clip.timelineStartMs + length(clip);

export function validateVideoComposition(project: Composition, context: z.RefinementCtx) {
  const issue: Issue = (message) => context.addIssue({ code: "custom", message });
  const assets = new Map(project.assets.map((asset) => [asset.nodeId, asset]));
  validateIdentities(project, assets, issue);
  const overlaps = validateTransitions(project, issue);
  for (const [index, clip] of project.videoTrack.clips.entries()) {
    validateVisualClip(clip, assets.get(clip.assetId), issue);
    validatePlacement(project.videoTrack.clips, index, overlaps, issue);
  }
  const duration = projectEndMs(project);
  for (const clip of project.audioTrack.clips)
    validateAudioClip(clip, assets.get(clip.assetId), duration, issue);
  validateAudioLanes(project.audioTrack.clips, issue);
  if (project.textTrack.items.some((item) => item.endMs > duration))
    issue("Timed text exceeds the project timeline");
}

function projectEndMs(project: Composition) {
  return project.videoTrack.clips.reduce((end, clip) => Math.max(end, timelineEnd(clip)), 0);
}

function validateVisualClip(clip: Clip, asset: Asset | undefined, issue: Issue) {
  if (!asset || asset.kind === "audio") {
    issue("A visual clip must use a video or image asset");
    return;
  }
  if (!validWindow(clip, asset)) issue("Invalid visual clip source window");
}

function validateAudioClip(clip: AudioClip, asset: Asset | undefined, duration: number, issue: Issue) {
  if (!asset || asset.kind !== "audio") {
    issue("An audio clip must use an audio asset");
    return;
  }
  if (!validWindow(clip, asset) || timelineEnd(clip) > duration)
    issue("Invalid audio clip source or timeline interval");
}

function validWindow(clip: Clip | AudioClip, asset: Asset) {
  const span = length(clip);
  return (
    span >= MIN_CLIP_MS &&
    clip.fadeInMs + clip.fadeOutMs <= span &&
    (asset.kind === "image" || clip.sourceOutMs <= asset.durationMs)
  );
}

function validateAudioLanes(clips: AudioClip[], issue: Issue) {
  const cursors = Array.from({ length: AUDIO_LANES }, () => 0);
  for (const clip of clips) {
    const cursor = cursors[clip.lane];
    if (cursor === undefined || clip.timelineStartMs < cursor) {
      issue("Audio clips in one lane cannot overlap");
      return;
    }
    cursors[clip.lane] = timelineEnd(clip);
  }
}

/** Outgoing transition length per clip index; zero where the cut is hard. */
function validateTransitions(project: Composition, issue: Issue) {
  const clips = project.videoTrack.clips;
  const overlaps = clips.map(() => 0);
  const sources = new Set<string>();
  for (const transition of project.transitions) {
    const index = clips.findIndex((clip) => clip.id === transition.fromClipId);
    if (index < 0 || index === clips.length - 1) {
      issue("A transition needs a clip and a following clip");
      continue;
    }
    if (sources.has(transition.fromClipId)) {
      issue("A clip boundary carries at most one transition");
      continue;
    }
    sources.add(transition.fromClipId);
    overlaps[index] = transition.durationMs;
  }
  for (const [index, clip] of clips.entries()) {
    const incoming = index > 0 ? (overlaps[index - 1] ?? 0) : 0;
    if (incoming + (overlaps[index] ?? 0) + MIN_CLIP_MS > length(clip))
      issue("Transition overlaps exceed available clip media");
  }
  return overlaps;
}

function validatePlacement(clips: Clip[], index: number, overlaps: number[], issue: Issue) {
  const previous = index > 0 ? clips[index - 1] : undefined;
  const clip = clips[index];
  if (!previous || !clip) return;
  const overlap = Math.max(0, timelineEnd(previous) - clip.timelineStartMs);
  if (clip.timelineStartMs <= previous.timelineStartMs || overlap !== (overlaps[index - 1] ?? 0))
    issue("Clip overlap must match the transition between the two clips");
}

function validateIdentities(project: Composition, assets: Map<string, Asset>, issue: Issue) {
  const ids = [
    ...project.videoTrack.clips,
    ...project.audioTrack.clips,
    ...project.transitions,
    ...project.textTrack.items,
  ].map(({ id }) => id);
  if (assets.size !== project.assets.length || new Set(ids).size !== ids.length)
    issue("Video project identities must be unique");
  if (project.assets[0]?.kind !== "video" || project.assets[0].sourceRevision !== project.sourceRevision)
    issue("Video project must retain its owning source");
}
