import { describe, expect, it } from "vitest";

import { videoEditProjectSchema } from "../../model/nodes/schema/video-edit";
import {
  closeGapBefore,
  duplicateClip,
  moveBounds,
  moveClip,
  removeClip,
  splitClip,
  swapClip,
} from "../edits/arrange";
import { insertMaterial } from "../edits/material";
import { addTextOverlay } from "../edits/text";
import { trimClip } from "../edits/trim";
import { createVideoEditProject, defaultTextStyle, type VideoAsset } from "../project";
import { canSplitAt, clipEndMs, compositionAt, projectDurationMs } from "../timing";

const second: VideoAsset = {
  nodeId: "second",
  sourceRevision: "second-revision",
  kind: "video",
  durationMs: 3_000,
  width: 1920,
  height: 1080,
};

function source() {
  return createVideoEditProject({
    nodeId: "source",
    sourceRevision: "revision",
    durationMs: 4_000,
    sourceWidth: 1280,
    sourceHeight: 720,
  });
}

function twoClips() {
  return insertMaterial(source(), second, 4_000);
}

describe("video timeline arrangement", () => {
  it("places a second source after the first without changing its media window", () => {
    const project = twoClips();
    expect(
      project.videoTrack.clips.map(({ assetId, timelineStartMs }) => [assetId, timelineStartMs]),
    ).toEqual([
      ["source", 0],
      ["second", 4_000],
    ]);
    expect(projectDurationMs(project)).toBe(7_000);
    expect(videoEditProjectSchema.safeParse(project).success).toBe(true);
  });

  it("splits a clip into two contiguous clips that keep the total duration", () => {
    const project = splitClip(source(), 1_500);
    expect(
      project.videoTrack.clips.map(({ sourceInMs, sourceOutMs, timelineStartMs }) => [
        sourceInMs,
        sourceOutMs,
        timelineStartMs,
      ]),
    ).toEqual([
      [0, 1_500, 0],
      [1_500, 4_000, 1_500],
    ]);
    expect(projectDurationMs(project)).toBe(4_000);
  });

  it("refuses a split that would leave less than a tenth of a second on either side", () => {
    const project = source();
    expect(canSplitAt(project, 50)).toBe(false);
    expect(canSplitAt(project, 3_950)).toBe(false);
    expect(canSplitAt(project, 2_000)).toBe(true);
    expect(splitClip(project, 50)).toBe(project);
  });

  it("ripples the timeline closed when a clip is removed", () => {
    const initial = twoClips();
    const project = removeClip(initial, initial.videoTrack.clips[0]!.id);
    expect(project.videoTrack.clips).toMatchObject([{ assetId: "second", timelineStartMs: 0 }]);
    expect(projectDurationMs(project)).toBe(3_000);
  });

  it("keeps the last clip and retains its source among the project assets", () => {
    const project = source();
    expect(removeClip(project, project.videoTrack.clips[0]!.id)).toBe(project);
  });

  it("inserts a duplicate directly after the clip and rolls later media forward", () => {
    const initial = addTextOverlay(twoClips(), 4_200, defaultTextStyle("Arial"), "Second");
    const project = duplicateClip(initial, initial.videoTrack.clips[0]!.id);
    expect(project.videoTrack.clips.map(({ assetId }) => assetId)).toEqual(["source", "source", "second"]);
    expect(project.videoTrack.clips[1]!.id).not.toBe(initial.videoTrack.clips[0]!.id);
    expect(project.textTrack.items[0]!.startMs).toBe(8_200);
    expect(initial.videoTrack.clips).toHaveLength(2);
  });

  it("exchanges neighbours while preserving the span the pair occupies", () => {
    const initial = twoClips();
    const project = swapClip(initial, initial.videoTrack.clips[0]!.id, 1);
    expect(
      project.videoTrack.clips.map(({ assetId, timelineStartMs }) => [assetId, timelineStartMs]),
    ).toEqual([
      ["second", 0],
      ["source", 3_000],
    ]);
    expect(projectDurationMs(project)).toBe(7_000);
  });

  it("allows a gap between clips and closes it on request", () => {
    const initial = twoClips();
    const clipId = initial.videoTrack.clips[1]!.id;
    expect(moveBounds(initial, clipId)).toMatchObject({ minMs: 4_000 });
    const moved = moveClip(initial, clipId, 5_500);
    expect(moved.videoTrack.clips[1]!.timelineStartMs).toBe(5_500);
    expect(projectDurationMs(moved)).toBe(8_500);
    expect(compositionAt(moved, 4_500).layers).toEqual([]);
    const closed = closeGapBefore(moved, clipId);
    expect(closed.videoTrack.clips[1]!.timelineStartMs).toBe(4_000);
  });

  it("clamps a dragged clip to its neighbour instead of overlapping it", () => {
    const initial = twoClips();
    const moved = moveClip(initial, initial.videoTrack.clips[1]!.id, 1_000);
    expect(moved.videoTrack.clips[1]!.timelineStartMs).toBe(4_000);
  });

  it("trims one edge without moving the opposite edge or the neighbours", () => {
    const initial = twoClips();
    const clip = initial.videoTrack.clips[0]!;
    const trimmed = trimClip(initial, clip.id, "end", 2_500);
    expect(trimmed.videoTrack.clips[0]).toMatchObject({ sourceInMs: 0, sourceOutMs: 2_500 });
    expect(clipEndMs(trimmed.videoTrack.clips[0]!)).toBe(2_500);
    expect(trimmed.videoTrack.clips[1]!.timelineStartMs).toBe(4_000);
    expect(trimClip(trimmed, clip.id, "end", 9_000).videoTrack.clips[0]!.sourceOutMs).toBe(4_000);
  });
});
