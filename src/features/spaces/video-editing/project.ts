import type { SpaceVideoPayload } from "../model/nodes/node-payload";
import { VIDEO_EDIT_PROJECT_VERSION } from "../model/nodes/schema/video-edit";

export type VideoEditProject = NonNullable<SpaceVideoPayload["videoEditProject"]>;
export type VideoOutput = VideoEditProject["output"];
export type VideoClip = VideoEditProject["videoTrack"]["clips"][number];
export type VideoAsset = VideoEditProject["assets"][number];
export type VideoAudioClip = VideoEditProject["audioTrack"]["clips"][number];
export type VideoTransition = VideoEditProject["transitions"][number];
export type VideoTransitionKind = VideoTransition["kind"];
export type VideoTransform = VideoClip["transform"];
export type VideoTextOverlay = VideoEditProject["textTrack"]["items"][number];
export type VideoTextStyle = VideoTextOverlay["style"];

const DEFAULT_BACKGROUND = "#000000";
export const STILL_IMAGE_MS = 4_000;
export const MIN_CLIP_MS = 100;

const DEFAULT_FRAME_RATE = 30;

interface VideoEditProjectSource {
  nodeId: string;
  sourceRevision: string;
  durationMs: number;
  sourceWidth: number;
  sourceHeight: number;
  /** Measured source cadence; the output starts at it instead of resampling the video. */
  frameRate?: VideoOutput["frameRate"] | undefined;
}

export function defaultTransform(): VideoTransform {
  return {
    x: 0,
    y: 0,
    scale: 1,
    rotation: 0,
    opacity: 1,
    fit: "contain",
    flipHorizontal: false,
    flipVertical: false,
  };
}

export function createVideoClip(
  assetId: string,
  window: { sourceOutMs: number; timelineStartMs: number },
): VideoClip {
  return {
    id: crypto.randomUUID(),
    assetId,
    sourceInMs: 0,
    sourceOutMs: window.sourceOutMs,
    timelineStartMs: window.timelineStartMs,
    transform: defaultTransform(),
    volume: 1,
    fadeInMs: 0,
    fadeOutMs: 0,
  };
}

export function createVideoEditProject(source: VideoEditProjectSource): VideoEditProject {
  return {
    version: VIDEO_EDIT_PROJECT_VERSION,
    sourceRevision: source.sourceRevision,
    assets: [
      {
        nodeId: source.nodeId,
        sourceRevision: source.sourceRevision,
        kind: "video",
        durationMs: source.durationMs,
        width: source.sourceWidth,
        height: source.sourceHeight,
      },
    ],
    output: {
      width: Math.max(2, source.sourceWidth - (source.sourceWidth % 2)),
      height: Math.max(2, source.sourceHeight - (source.sourceHeight % 2)),
      frameRate: source.frameRate ?? DEFAULT_FRAME_RATE,
      background: DEFAULT_BACKGROUND,
    },
    videoTrack: {
      clips: [createVideoClip(source.nodeId, { sourceOutMs: source.durationMs, timelineStartMs: 0 })],
    },
    transitions: [],
    audioTrack: { clips: [] },
    textTrack: { items: [] },
  };
}

export function defaultTextStyle(family: string): VideoTextStyle {
  return {
    family,
    weight: 600,
    italic: false,
    size: 48,
    color: "#FFFFFF",
    align: "center",
    decoration: "outline",
    decorationStrength: 3,
    decorationColor: "#000000",
  };
}

export function createTextOverlay(
  startMs: number,
  endMs: number,
  style: VideoTextStyle,
  text: string,
): VideoTextOverlay {
  return { id: crypto.randomUUID(), startMs, endMs, text, x: 0.5, y: 0.82, style };
}
