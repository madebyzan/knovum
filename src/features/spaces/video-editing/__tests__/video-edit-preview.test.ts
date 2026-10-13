import { describe, expect, it } from "vitest";

import { insertMaterial } from "../edits/material";
import { addTextOverlay } from "../edits/text";
import { applyTransition } from "../edits/transition";
import { setClipTransform } from "../edits/trim";
import { planPreview } from "../engine/layer-plan";
import { HARD_SEEK_MS, playbackRateFor } from "../engine/media-sync";
import { fitRatio, layerGeometry, scaleFromSize } from "../geometry";
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

describe("preview plan", () => {
  it("describes a single layer with its own source position outside a transition", () => {
    const plan = planPreview(project(), 1_000);
    expect(plan.layers).toHaveLength(1);
    expect(plan.layers[0]).toMatchObject({ sourceMs: 1_000, gain: 1 });
    expect(plan.transition).toBeUndefined();
  });

  it("weights the two layers of a transition so sound and picture match the export", () => {
    const initial = project();
    const mixed = applyTransition(initial, initial.videoTrack.clips[0]!.id, "dissolve", 1_000);
    const plan = planPreview(mixed, 3_250);
    expect(plan.transition).toMatchObject({ kind: "dissolve", progress: 0.25 });
    expect(plan.layers.map(({ gain }) => Number(gain.toFixed(4)))).toEqual([0.75, 0.25]);
    expect(plan.layers.map(({ sourceMs }) => sourceMs)).toEqual([3_250, 250]);
  });

  it("plans the media of an upcoming cut so the next clip is already decoded", () => {
    const plan = planPreview(project(), 3_200);
    expect(plan.preroll.map(({ asset }) => asset.nodeId)).toEqual(["image"]);
    expect(planPreview(project(), 1_000).preroll).toEqual([]);
  });

  it("includes the music and text that cover the position", () => {
    const initial = addTextOverlay(
      insertMaterial(project(), music, 0),
      500,
      defaultTextStyle("Arial"),
      "Title",
    );
    const plan = planPreview(initial, 600);
    expect(plan.audio.map(({ clip }) => clip.assetId)).toEqual(["music"]);
    expect(plan.texts.map(({ text }) => text)).toEqual(["Title"]);
    expect(planPreview(initial, 4_000).texts).toEqual([]);
  });

  it("shows nothing but the background inside a gap", () => {
    const initial = project();
    const plan = planPreview({ ...initial, videoTrack: { clips: [initial.videoTrack.clips[1]!] } }, 1_000);
    expect(plan.layers).toEqual([]);
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
});

describe("playback clock", () => {
  it("trims the playback rate towards the clock and never exceeds a tolerable nudge", () => {
    expect(playbackRateFor(0)).toBe(1);
    expect(playbackRateFor(-20)).toBeCloseTo(1.032, 5);
    expect(playbackRateFor(20)).toBeCloseTo(0.968, 5);
    expect(playbackRateFor(-10_000)).toBeCloseTo(1.06, 5);
    expect(playbackRateFor(HARD_SEEK_MS)).toBeCloseTo(0.94, 5);
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
