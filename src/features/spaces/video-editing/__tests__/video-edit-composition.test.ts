import { describe, expect, it } from "vitest";

import { videoEditProjectSchema } from "../../model/nodes/schema/video-edit";
import { replaceClip, setTransition, splitClip } from "../clip-edits";
import { duplicateClip, moveClip } from "../clip-order";
import { appendAsset } from "../material-edits";
import { addTextOverlay, createVideoEditProject } from "../project";
import { canSplitClip, locateTimelineFrames, projectDurationMs } from "../timing";

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
  return appendAsset(source(), {
    nodeId: "second",
    sourceRevision: "second-revision",
    kind: "video",
    durationMs: 3_000,
    width: 1920,
    height: 1080,
  });
}

describe("video composition", () => {
  it("inserts another managed source without replacing the first", () => {
    const project = twoClips();
    expect(project.assets.map(({ nodeId }) => nodeId)).toEqual(["source", "second"]);
    expect(project.videoTrack.clips.map(({ assetId }) => assetId)).toEqual(["source", "second"]);
    expect(projectDurationMs(project)).toBe(7_000);
    expect(videoEditProjectSchema.safeParse(project).success).toBe(true);
  });
  it("subtracts a transition overlap and samples both source intervals", () => {
    const initial = twoClips();
    const project = setTransition(initial, initial.videoTrack.clips[0]!.id, {
      kind: "dissolve",
      durationMs: 1_000,
    });
    expect(projectDurationMs(project)).toBe(6_000);
    const frames = locateTimelineFrames(project, 3_500);
    expect(frames.map(({ clip, offsetMs }) => [clip.assetId, offsetMs])).toEqual([
      ["source", 3_500],
      ["second", 500],
    ]);
    expect(frames[1]!.progress).toBe(0.5);
    expect(locateTimelineFrames(project, 4_000).map(({ clip }) => clip.assetId)).toEqual(["second"]);
  });
  it("rejects overlapping transitions that consume an entire clip", () => {
    const initial = twoClips();
    expect(() =>
      setTransition(initial, initial.videoTrack.clips[0]!.id, { kind: "dissolve", durationMs: 3_000 }),
    ).toThrow("Transition overlaps exceed available clip media");
    const malformed = structuredClone(initial);
    malformed.videoTrack.clips[0]!.transition = { kind: "dissolve", durationMs: 3_000 };
    expect(videoEditProjectSchema.safeParse(malformed).success).toBe(false);
  });
  it("duplicates and moves a clip as immutable, independently editable items", () => {
    const initial = twoClips();
    const duplicate = duplicateClip(initial, initial.videoTrack.clips[0]!.id);
    expect(duplicate.videoTrack.clips).toHaveLength(3);
    expect(duplicate.videoTrack.clips[1]!.id).not.toBe(initial.videoTrack.clips[0]!.id);
    const moved = moveClip(duplicate, duplicate.videoTrack.clips[1]!.id, 1);
    expect(moved.videoTrack.clips.map(({ assetId }) => assetId)).toEqual(["source", "second", "source"]);
    expect(initial.videoTrack.clips).toHaveLength(2);
  });
  it("rejects obsolete projects and forged asset references", () => {
    expect(
      videoEditProjectSchema.safeParse({
        version: 1,
        sourceRevision: "old",
        sourceWidth: 1280,
        sourceHeight: 720,
        videoTrack: { clips: [] },
        textTrack: { items: [] },
        transform: { rotation: 0, flipHorizontal: false, flipVertical: false },
      }).success,
    ).toBe(false);
    const forged = source();
    forged.videoTrack.clips[0]!.assetId = "unknown";
    expect(videoEditProjectSchema.safeParse(forged).success).toBe(false);
  });
});

it("removes a transition only when its connected pair changes during reorder", () => {
  const initial = twoClips();
  const mixed = setTransition(initial, initial.videoTrack.clips[0]!.id, {
    kind: "dissolve",
    durationMs: 500,
  });
  const moved = moveClip(mixed, mixed.videoTrack.clips[0]!.id, 1);
  expect(moved.videoTrack.clips.map(({ transition }) => transition)).toEqual([null, null]);
  expect(projectDurationMs(moved)).toBe(7_000);
});

it("splits at an integer source time after stepping a 30 fps output frame", () => {
  const initial = source();
  const next = splitClip(initial, 1000 / 3);
  expect(next.videoTrack.clips[0]!.endMs).toBe(333);
  expect(next.videoTrack.clips[1]!.startMs).toBe(333);
  expect(projectDurationMs(next)).toBe(4000);
});

it("keeps fade envelopes intact when deciding split availability", () => {
  const initial = source();
  initial.videoTrack.clips[0]!.fadeInMs = 1000;
  initial.videoTrack.clips[0]!.fadeOutMs = 1000;
  expect(canSplitClip(initial, 500)).toBe(false);
  expect(canSplitClip(initial, 3500)).toBe(false);
  expect(canSplitClip(initial, 2000)).toBe(true);
});

it("retimes tracks once when both ends of a clip change", () => {
  const initial = addTextOverlay(source(), 3500, "Arial", "End");
  initial.videoTrack.clips[0]!.startMs = 1000;
  initial.videoTrack.clips[0]!.endMs = 4000;
  initial.textTrack.items[0]!.startMs = 2000;
  initial.textTrack.items[0]!.endMs = 3000;
  const next = replaceClip(initial, { ...initial.videoTrack.clips[0]!, startMs: 0, endMs: 3500 });
  expect(next.textTrack.items[0]).toMatchObject({ startMs: 3000, endMs: 3500 });
});

it("moves text on the next clip when duplicating a transitioned clip", () => {
  const initial = twoClips();
  const mixed = addTextOverlay(
    setTransition(initial, initial.videoTrack.clips[0]!.id, {
      kind: "dissolve",
      durationMs: 500,
    }),
    3500,
    "Arial",
    "Next",
  );
  const duplicate = duplicateClip(mixed, mixed.videoTrack.clips[0]!.id);
  expect(duplicate.textTrack.items[0]!.startMs).toBe(8000);
});
