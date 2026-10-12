import { describe, expect, it } from "vitest";

import { setTransition } from "../clip-edits";
import { appendAsset } from "../material-edits";
import { previewFrames, previewGeometry, previewTimeMs } from "../preview-composition";
import { createVideoEditProject } from "../project";

function project() {
  const source = createVideoEditProject({
    nodeId: "source",
    sourceRevision: "revision",
    durationMs: 4_000,
    sourceWidth: 1280,
    sourceHeight: 720,
  });
  return appendAsset(source, {
    nodeId: "image",
    sourceRevision: "image-revision",
    kind: "image",
    durationMs: 0,
    width: 720,
    height: 1280,
  });
}
describe("video preview composition", () => {
  it("uses complementary visual and audio weights during a dissolve", () => {
    const initial = project();
    const mixed = setTransition(initial, initial.videoTrack.clips[0]!.id, {
      kind: "dissolve",
      durationMs: 1_000,
    });
    const frames = previewFrames(mixed, 3_250);
    expect(frames.map(({ opacity }) => opacity)).toEqual([0.75, 0.25]);
    expect(frames.map(({ gain }) => gain)).toEqual([0.75, 0.25]);
  });
  it("fits and zooms oriented media against the output frame", () => {
    const initial = project();
    const clip = initial.videoTrack.clips[1]!;
    const image = initial.assets[1]!;
    expect(previewGeometry(image, clip.transform, initial.output)).toMatchObject({ width: 405, height: 720 });
    expect(previewGeometry(image, { ...clip.transform, rotation: 90 }, initial.output)).toMatchObject({
      width: 720,
      height: 1280,
    });
  });
});

it("shows the last exported output frame when the playhead reaches the project end", () => {
  const initial = project();
  expect(previewTimeMs(initial, 7000)).toBeCloseTo(6966.6666667);
  expect(previewFrames(initial, 7000)[0]!.offsetMs).toBeCloseTo(2966.6666667);
});
