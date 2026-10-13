import { describe, expect, it } from "vitest";

import { insertMaterial } from "../edits/material";
import { addTextOverlay } from "../edits/text";
import { applyTransition } from "../edits/transition";
import { setClipTransform } from "../edits/trim";
import { fitRatio, layerGeometry, scaleFromSize } from "../geometry";
import { buildScene } from "../playback/scene-plan";
import { createVideoEditProject, defaultTextStyle, type VideoAsset } from "../project";
import { lastFrameMs, stepFrame } from "../timing";

const image: VideoAsset = {
  nodeId: "image",
  sourceRevision: "image-revision",
  kind: "image",
  durationMs: 0,
  width: 720,
  height: 1280,
};
const music: VideoAsset = {
  nodeId: "music",
  sourceRevision: "music-revision",
  kind: "audio",
  durationMs: 30_000,
  width: 0,
  height: 0,
};

const picture = {} as CanvasImageSource;
const decoded = () => picture;
const decoding = () => undefined;

function project() {
  const source = createVideoEditProject({
    nodeId: "source",
    sourceRevision: "revision",
    durationMs: 4_000,
    sourceWidth: 1280,
    sourceHeight: 720,
  });
  return insertMaterial(source, image, 4_000);
}

describe("preview scene", () => {
  it("describes a single layer with its own source position outside a transition", () => {
    const scene = buildScene(project(), 1_000, (_asset, sourceMs) => {
      expect(sourceMs).toBe(1_000);
      return picture;
    });
    expect(scene.layers).toHaveLength(1);
    expect(scene.layers[0]).toMatchObject({ fade: 1, image: picture });
    expect(scene.transition).toBeUndefined();
  });

  it("carries both layers of a transition with the progress the export uses", () => {
    const initial = project();
    const mixed = applyTransition(initial, initial.videoTrack.clips[0]!.id, "dissolve", 1_000);
    const scene = buildScene(mixed, 3_250, decoded);
    expect(scene.transition).toMatchObject({ kind: "dissolve", progress: 0.25 });
    expect(scene.layers.map(({ fade }) => Number(fade.toFixed(4)))).toEqual([1, 1]);
  });

  it("omits a layer that is still decoding, and the transition it would be part of", () => {
    const initial = project();
    const mixed = applyTransition(initial, initial.videoTrack.clips[0]!.id, "dissolve", 1_000);
    const scene = buildScene(mixed, 3_250, (asset) => (asset.kind === "video" ? picture : undefined));
    expect(scene.layers).toHaveLength(1);
    expect(scene.transition).toBeUndefined();
  });

  it("includes the text that covers the position and nothing else", () => {
    const initial = addTextOverlay(
      insertMaterial(project(), music, 0),
      500,
      defaultTextStyle("Arial"),
      "Title",
    );
    expect(buildScene(initial, 600, decoded).texts.map(({ text }) => text)).toEqual(["Title"]);
    expect(buildScene(initial, 4_000, decoded).texts).toEqual([]);
  });

  it("shows nothing but the background inside a gap", () => {
    const initial = project();
    const gap = { ...initial, videoTrack: { clips: [initial.videoTrack.clips[1]!] } };
    expect(buildScene(gap, 1_000, decoded).layers).toEqual([]);
    expect(buildScene(initial, 1_000, decoding).layers).toEqual([]);
  });
});

describe("preview framing", () => {
  it("fits portrait media into a landscape frame and covers it on demand", () => {
    const initial = project();
    const clip = initial.videoTrack.clips[1]!;
    expect(layerGeometry(image, clip.transform, initial.output)).toMatchObject({
      width: 405,
      height: 720,
      centerX: 640,
      centerY: 360,
    });
    expect(fitRatio(image, initial.output, "cover")).toBeCloseTo(1.7778, 4);
  });

  it("keeps a dragged and scaled layer consistent with the size on screen", () => {
    const initial = project();
    const clip = initial.videoTrack.clips[1]!;
    const moved = setClipTransform(initial, clip.id, { x: 0.25, scale: 2 });
    const geometry = layerGeometry(image, moved.videoTrack.clips[1]!.transform, moved.output);
    expect(geometry).toMatchObject({ width: 810, centerX: 960 });
    expect(scaleFromSize(image, moved.output, "contain", geometry.width)).toBeCloseTo(2, 5);
  });

  it("steps whole output frames and stops on the last displayable frame", () => {
    const initial = project();
    let position = 0;
    for (let frame = 0; frame < 30; frame += 1) position = stepFrame(initial, position, 1);
    expect(position).toBeCloseTo(1_000, 8);
    expect(stepFrame(initial, 0, -1)).toBe(0);
    expect(lastFrameMs(initial)).toBeCloseTo(7_966.6667, 4);
    expect(stepFrame(initial, 8_000, 1)).toBe(8_000);
  });
});
