import type { VideoAsset, VideoAudioClip, VideoClip, VideoEditProject } from "../project";
import { clipHeadroom, clipLengthMs } from "../timing";
import type { AudioVoice } from "./audio-mixer";

export type BufferResolver = (asset: VideoAsset) => AudioBuffer | undefined;

function voice(
  clip: VideoClip | VideoAudioClip,
  buffer: AudioBuffer,
  fades: { inMs: number; outMs: number },
): AudioVoice {
  return {
    key: clip.id,
    buffer,
    sourceInMs: clip.sourceInMs,
    lengthMs: clipLengthMs(clip),
    timelineStartMs: clip.timelineStartMs,
    volume: clip.volume,
    fadeInMs: fades.inMs,
    fadeOutMs: fades.outMs,
  };
}

/**
 * Every sound the project plays, as voices the mixer can schedule once. A visual clip's fades are
 * widened by the transition overlap it shares with its neighbour, which is the same crossfade the
 * export applies.
 */
export function buildVoices(project: VideoEditProject, resolve: BufferResolver): AudioVoice[] {
  const assets = new Map(project.assets.map((asset) => [asset.nodeId, asset]));
  const bufferOf = (assetId: string) => {
    const asset = assets.get(assetId);
    return asset ? resolve(asset) : undefined;
  };
  const clips = project.videoTrack.clips.flatMap((clip) => {
    const buffer = bufferOf(clip.assetId);
    if (!buffer) return [];
    const { incomingMs, outgoingMs } = clipHeadroom(project, clip);
    const inMs = Math.max(clip.fadeInMs, incomingMs);
    return [voice(clip, buffer, { inMs, outMs: Math.max(clip.fadeOutMs, outgoingMs) })];
  });
  const music = project.audioTrack.clips.flatMap((clip) => {
    const buffer = bufferOf(clip.assetId);
    return buffer ? [voice(clip, buffer, { inMs: clip.fadeInMs, outMs: clip.fadeOutMs })] : [];
  });
  return [...clips, ...music];
}
