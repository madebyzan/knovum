import type { VideoAudioClip, VideoClip, VideoEditProject } from "../project";
import { clipHeadroom, clipLengthMs } from "../timing";
import type { AudioVoice } from "./audio-mixer";

/**
 * Every sound the project plays, as voices the mixer streams. A still image has none. A visual
 * clip's fades are widened by the transition overlap it shares with its neighbour, which is the
 * same crossfade the export applies.
 */
export function buildVoices(project: VideoEditProject): AudioVoice[] {
  const assetByNodeId = new Map(project.assets.map((asset) => [asset.nodeId, asset]));
  const voice = (clip: VideoClip | VideoAudioClip, fades: { inMs: number; outMs: number }) => {
    const asset = assetByNodeId.get(clip.assetId);
    if (!asset || asset.kind === "image") return [];
    return [
      {
        key: clip.id,
        asset,
        sourceInMs: clip.sourceInMs,
        lengthMs: clipLengthMs(clip),
        timelineStartMs: clip.timelineStartMs,
        volume: clip.volume,
        fadeInMs: fades.inMs,
        fadeOutMs: fades.outMs,
      },
    ];
  };
  const clips = project.videoTrack.clips.flatMap((clip) => {
    const { incomingMs, outgoingMs } = clipHeadroom(project, clip);
    return voice(clip, {
      inMs: Math.max(clip.fadeInMs, incomingMs),
      outMs: Math.max(clip.fadeOutMs, outgoingMs),
    });
  });
  const music = project.audioTrack.clips.flatMap((clip) =>
    voice(clip, { inMs: clip.fadeInMs, outMs: clip.fadeOutMs }),
  );
  return [...clips, ...music];
}
