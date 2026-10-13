import type {
  VideoAsset,
  VideoAudioClip,
  VideoClip,
  VideoEditProject,
  VideoTextOverlay,
  VideoTransitionKind,
} from "../project";
import { audioClipsAt, clipAudioGain, compositionAt, fadeGain, textItemsAt } from "../timing";

const PREROLL_MS = 1_200;

interface PlannedLayer {
  clip: VideoClip;
  asset: VideoAsset;
  sourceMs: number;
  /** Visual fade of the clip itself. */
  fade: number;
  gain: number;
}

interface PlannedAudio {
  clip: VideoAudioClip;
  asset: VideoAsset;
  sourceMs: number;
  gain: number;
}

export interface PreviewPlan {
  layers: PlannedLayer[];
  transition?: { kind: VideoTransitionKind; progress: number } | undefined;
  audio: PlannedAudio[];
  texts: VideoTextOverlay[];
  /** Media to load and park ahead of time so a cut never waits for a decoder. */
  preroll: { asset: VideoAsset; sourceMs: number }[];
}

/**
 * Pure description of what the preview must show and prepare at one timeline position. Keeping this
 * separate from the engine makes the cut, transition and pre-roll rules directly testable.
 */
export function planPreview(project: VideoEditProject, timeMs: number): PreviewPlan {
  const assets = new Map(project.assets.map((asset) => [asset.nodeId, asset]));
  const composition = compositionAt(project, timeMs);
  const layers = composition.layers.flatMap((layer) => {
    const asset = assets.get(layer.clip.assetId);
    if (!asset) return [];
    return [
      {
        clip: layer.clip,
        asset,
        sourceMs: layer.sourceMs,
        fade: layer.fade,
        gain: clipAudioGain(project, layer.clip, layer.offsetMs),
      },
    ];
  });
  const audio = audioClipsAt(project, timeMs).flatMap((clip) => {
    const asset = assets.get(clip.assetId);
    if (!asset) return [];
    const offsetMs = timeMs - clip.timelineStartMs;
    return [
      {
        clip,
        asset,
        sourceMs: clip.sourceInMs + offsetMs,
        gain: clip.volume * fadeGain(clip, offsetMs),
      },
    ];
  });
  return {
    layers,
    transition: composition.transition
      ? {
          kind: composition.transition.transition.kind,
          progress: composition.transition.progress,
        }
      : undefined,
    audio,
    texts: textItemsAt(project, timeMs),
    preroll: upcoming(project, assets, timeMs),
  };
}

function upcoming(
  project: VideoEditProject,
  assets: Map<string, VideoAsset>,
  timeMs: number,
): { asset: VideoAsset; sourceMs: number }[] {
  const window = timeMs + PREROLL_MS;
  const clips = project.videoTrack.clips.filter(
    (clip) => clip.timelineStartMs > timeMs && clip.timelineStartMs <= window,
  );
  const tracks = project.audioTrack.clips.filter(
    (clip) => clip.timelineStartMs > timeMs && clip.timelineStartMs <= window,
  );
  return [...clips, ...tracks].flatMap((clip) => {
    const asset = assets.get(clip.assetId);
    return asset ? [{ asset, sourceMs: clip.sourceInMs }] : [];
  });
}
