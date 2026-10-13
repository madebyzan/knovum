import { describe, expect, it } from "vitest";

import { insertMaterial } from "../edits/material";
import { createVideoEditProject, type VideoAsset } from "../project";
import { stepTrimFrame } from "../timeline/frame-step";
import {
  clampPxPerSecond,
  fitPxPerSecond,
  MAX_PX_PER_SECOND,
  pxToTime,
  scrollToReveal,
  tickStepMs,
  timelineWidth,
  timeToPx,
  visibleTicks,
  zoomAroundPointer,
} from "../timeline/scale";
import { snapTime } from "../timeline/snapping";
import { clipEndMs } from "../timing";

const second: VideoAsset = {
  nodeId: "second",
  sourceRevision: "second-revision",
  kind: "video",
  durationMs: 3_000,
  width: 1280,
  height: 720,
};

function project() {
  const source = createVideoEditProject({
    nodeId: "source",
    sourceRevision: "revision",
    durationMs: 4_000,
    sourceWidth: 1280,
    sourceHeight: 720,
  });
  return insertMaterial(source, second, 4_000);
}

describe("timeline scale", () => {
  it("converts between time and pixels at the current scale", () => {
    expect(timeToPx(2_500, 60)).toBe(150);
    expect(pxToTime(150, 60)).toBe(2_500);
    expect(clampPxPerSecond(10_000)).toBe(MAX_PX_PER_SECOND);
  });

  it("fits the whole project with trailing room to work past the last clip", () => {
    const scale = fitPxPerSecond(7_000, 900);
    expect(scale).toBeCloseTo(100, 5);
    expect(timelineWidth(7_000, scale)).toBeCloseTo(900, 5);
  });

  it("keeps ruler labels readable by choosing a coarser step as the scale shrinks", () => {
    expect(tickStepMs(200)).toBe(500);
    expect(tickStepMs(10)).toBe(10_000);
    const ticks = visibleTicks({
      pxPerSecond: 100,
      scrollLeft: 0,
      viewportWidth: 300,
      durationMs: 7_000,
    });
    expect(ticks[0]).toBe(0);
    expect(ticks.at(-1)).toBeLessThanOrEqual(9_000);
  });

  it("holds the time under the pointer while zooming", () => {
    const zoomed = zoomAroundPointer({ pxPerSecond: 60, factor: 2, scrollLeft: 60, pointerX: 120 });
    expect(zoomed.pxPerSecond).toBe(120);
    expect(pxToTime(zoomed.scrollLeft + 120, zoomed.pxPerSecond)).toBeCloseTo(3_000, 5);
  });

  it("follows the playhead only once it leaves the comfortable band", () => {
    const inside = scrollToReveal({
      timeMs: 2_000,
      pxPerSecond: 100,
      scrollLeft: 100,
      viewportWidth: 400,
    });
    expect(inside).toBe(100);
    expect(scrollToReveal({ timeMs: 6_000, pxPerSecond: 100, scrollLeft: 100, viewportWidth: 400 })).toBe(
      400,
    );
  });
});

describe("timeline snapping", () => {
  it("snaps a dragged time to clip edges and to the playhead", () => {
    const initial = project();
    expect(snapTime({ project: initial, timeMs: 3_960, pxPerSecond: 60 })).toMatchObject({
      timeMs: 4_000,
      snappedTo: 4_000,
    });
    expect(snapTime({ project: initial, timeMs: 1_980, pxPerSecond: 60, playheadMs: 2_000 })).toMatchObject({
      timeMs: 2_000,
    });
  });

  it("falls back to the output frame grid and honours a disabled modifier", () => {
    const initial = project();
    expect(snapTime({ project: initial, timeMs: 1_510, pxPerSecond: 60 }).timeMs).toBe(1_500);
    expect(snapTime({ project: initial, timeMs: 3_960, pxPerSecond: 60, disabled: true })).toEqual({
      timeMs: 3_960,
    });
  });

  it("ignores the edge being dragged so a clip does not snap to itself", () => {
    const initial = project();
    const snapped = snapTime({
      project: initial,
      timeMs: 4_010,
      pxPerSecond: 60,
      exclude: [4_000],
    });
    expect(snapped.snappedTo).toBeUndefined();
  });
});

describe("keyboard trimming", () => {
  it("moves one edge by exactly one output frame and stops at the limit", () => {
    const initial = project();
    const clip = initial.videoTrack.clips[0]!;
    const trimmed = stepTrimFrame(initial, clip, "end", -1);
    expect(clipEndMs(trimmed!.videoTrack.clips[0]!)).toBe(3_967);
    expect(stepTrimFrame(initial, clip, "start", -1)).toBeUndefined();
    const started = stepTrimFrame(initial, clip, "start", 1);
    expect(started!.videoTrack.clips[0]).toMatchObject({ sourceInMs: 33, timelineStartMs: 33 });
  });
});
