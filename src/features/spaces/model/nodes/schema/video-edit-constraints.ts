import type { z } from "zod";

import type { videoAssetSchema, videoAudioClipSchema, videoClipSchema } from "./video-edit-media";
type Asset = z.infer<typeof videoAssetSchema>;
type Clip = z.infer<typeof videoClipSchema>;
type AudioClip = z.infer<typeof videoAudioClipSchema>;
interface Composition {
  sourceRevision: string;
  assets: Asset[];
  videoTrack: { clips: Clip[] };
  audioTrack: { clips: AudioClip[] };
  textTrack: { items: { id: string; startMs: number; endMs: number }[] };
}
export function validateVideoComposition(project: Composition, context: z.RefinementCtx) {
  const assets = new Map(project.assets.map((asset) => [asset.nodeId, asset]));
  const issue = (message: string) => context.addIssue({ code: "custom", message });
  validateIdentities(project, assets, issue);
  for (const [index, clip] of project.videoTrack.clips.entries()) {
    if (!validVisualClip(clip, assets.get(clip.assetId))) issue("Invalid visual clip source or interval");
    if (!validTransition(project.videoTrack.clips, clip, index))
      issue("Transition overlaps exceed available clip media");
  }
  const duration = project.videoTrack.clips.reduce(
    (sum, clip) => sum + clip.endMs - clip.startMs - (clip.transition?.durationMs ?? 0),
    0,
  );
  for (const clip of project.audioTrack.clips)
    if (!validAudioClip(clip, assets.get(clip.assetId), duration))
      issue("Invalid audio clip source or timeline interval");
  if (project.textTrack.items.some((item) => item.endMs <= item.startMs || item.endMs > duration))
    issue("Timed text exceeds the project timeline");
}
function validVisualClip(clip: Clip, asset: Asset | undefined) {
  return !!asset && asset.kind !== "audio" && validInterval(clip, asset);
}
function validAudioClip(clip: AudioClip, asset: Asset | undefined, duration: number) {
  return (
    !!asset &&
    asset.kind === "audio" &&
    validInterval(clip, asset) &&
    clip.timelineStartMs + clip.endMs - clip.startMs <= duration
  );
}
function validTransition(clips: Clip[], clip: Clip, index: number) {
  const incoming = clips[index - 1]?.transition?.durationMs ?? 0;
  const outgoing = clip.transition?.durationMs ?? 0;
  return (
    incoming + outgoing <= clip.endMs - clip.startMs - 100 && (!clip.transition || index !== clips.length - 1)
  );
}
function validInterval(clip: Clip | AudioClip, asset: Asset) {
  const duration = clip.endMs - clip.startMs;
  return (
    duration >= 100 &&
    clip.endMs <= 86_400_000 &&
    (asset.kind === "image" || clip.endMs <= asset.durationMs) &&
    clip.fadeInMs + clip.fadeOutMs <= duration
  );
}

function validateIdentities(
  project: Composition,
  assets: Map<string, Asset>,
  issue: (message: string) => void,
) {
  const ids = [...project.videoTrack.clips, ...project.audioTrack.clips, ...project.textTrack.items].map(
    ({ id }) => id,
  );
  if (assets.size !== project.assets.length || new Set(ids).size !== ids.length)
    issue("Video project identities must be unique");
  if (project.assets[0]?.kind !== "video" || project.assets[0].sourceRevision !== project.sourceRevision)
    issue("Video project must retain its owning source");
}
