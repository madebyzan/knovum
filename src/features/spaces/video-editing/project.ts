import { clamp } from "es-toolkit";

import type { SpaceVideoPayload } from "../model/nodes/node-payload";
import { projectDurationMs } from "./timing";

export type VideoEditProject = NonNullable<SpaceVideoPayload["videoEditProject"]>;
export type VideoClip = VideoEditProject["videoTrack"]["clips"][number];
export type VideoAsset = VideoEditProject["assets"][number];
export type VideoAudioClip = VideoEditProject["audioTrack"]["clips"][number];
export type VideoTransition = NonNullable<VideoClip["transition"]>;
export type VideoTextOverlay = VideoEditProject["textTrack"]["items"][number];
export type VideoTextStyle = VideoTextOverlay["style"];

interface VideoEditProjectSource {
  nodeId: string;
  sourceRevision: string;
  durationMs: number;
  sourceWidth: number;
  sourceHeight: number;
}
function defaultVideoTransform(): VideoClip["transform"] {
  return { rotation: 0, flipHorizontal: false, flipVertical: false, fit: "contain", zoom: 1 };
}
export function createVideoClip(assetId: string, durationMs: number): VideoClip {
  return {
    id: crypto.randomUUID(),
    assetId,
    startMs: 0,
    endMs: durationMs,
    volume: 1,
    fadeInMs: 0,
    fadeOutMs: 0,
    transform: defaultVideoTransform(),
    transition: null,
  };
}
export function createVideoEditProject(source: VideoEditProjectSource): VideoEditProject {
  return {
    version: 1,
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
      frameRate: 30,
    },
    videoTrack: { clips: [createVideoClip(source.nodeId, source.durationMs)] },
    audioTrack: { clips: [] },
    textTrack: { items: [] },
  };
}
export function addTextOverlay(
  project: VideoEditProject,
  startMs: number,
  family: string,
  text: string,
): VideoEditProject {
  const durationMs = projectDurationMs(project);
  const boundedStart = clamp(Math.round(startMs), 0, Math.max(0, durationMs - 100));
  const item: VideoTextOverlay = {
    id: crypto.randomUUID(),
    startMs: boundedStart,
    endMs: clamp(boundedStart + 3_000, boundedStart + 100, durationMs),
    text,
    placement: "bottom",
    style: { family, weight: 400, size: 30, color: "#FFFFFF", preset: "outline" },
  };
  return { ...project, textTrack: { items: [...project.textTrack.items, item] } };
}
