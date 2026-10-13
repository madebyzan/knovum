import { describe, expect, it } from "vitest";

import { removeClip } from "../edits/arrange";
import { removeAudioClip, setAudioClipLane } from "../edits/audio";
import { assetOf, pruneAssets } from "../edits/commit";
import { freeLane, insertMaterial } from "../edits/material";
import { createVideoEditProject, STILL_IMAGE_MS, type VideoAsset } from "../project";
import { clipEndMs, clipLengthMs, projectDurationMs } from "../timing";

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

function source() {
  return createVideoEditProject({
    nodeId: "primary",
    sourceRevision: "primary-revision",
    durationMs: 4_000,
    sourceWidth: 1280,
    sourceHeight: 720,
  });
}

describe("video editor materials", () => {
  it("gives a still image a default on-screen length and rolls the timeline forward", () => {
    const project = insertMaterial(source(), image, 0);
    expect(project.videoTrack.clips[0]).toMatchObject({ assetId: "image", timelineStartMs: 0 });
    expect(clipLengthMs(project.videoTrack.clips[0]!)).toBe(STILL_IMAGE_MS);
    expect(projectDurationMs(project)).toBe(4_000 + STILL_IMAGE_MS);
  });

  it("inserts visual media at the boundary of the clip under the playhead", () => {
    const project = insertMaterial(source(), image, 2_500);
    expect(
      project.videoTrack.clips.map(({ assetId, timelineStartMs }) => [assetId, timelineStartMs]),
    ).toEqual([
      ["image", 0],
      ["primary", STILL_IMAGE_MS],
    ]);
  });

  it("takes music into the first free lane and trims it to the timeline", () => {
    const project = insertMaterial(source(), music, 1_000);
    const clip = project.audioTrack.clips[0]!;
    expect(clip).toMatchObject({ assetId: "music", lane: 0, timelineStartMs: 1_000 });
    expect(clipEndMs(clip)).toBe(4_000);
    const second = insertMaterial(project, music, 1_000);
    expect(second.audioTrack.clips[1]!.lane).toBe(1);
    expect(freeLane(second, 1_000, 4_000)).toBe(2);
  });

  it("refuses an overlapping lane change and accepts a free one", () => {
    const project = insertMaterial(insertMaterial(source(), music, 0), music, 0);
    const [first, other] = project.audioTrack.clips;
    expect(setAudioClipLane(project, other!.id, first!.lane)).toBe(project);
    expect(setAudioClipLane(project, other!.id, 3).audioTrack.clips[1]!.lane).toBe(3);
  });

  it("rejects material whose source changed since the project referenced it", () => {
    const project = insertMaterial(source(), image, 0);
    expect(() => insertMaterial(project, { ...image, sourceRevision: "changed" }, 0)).toThrow(
      "Video material changed",
    );
  });

  it("releases a source once its last clip is gone but keeps the owning source", () => {
    const composed = insertMaterial(insertMaterial(source(), image, 4_000), music, 0);
    const withoutImage = removeClip(composed, composed.videoTrack.clips[1]!.id);
    expect(withoutImage.assets.map(({ nodeId }) => nodeId)).toEqual(["primary", "music"]);
    const silent = removeAudioClip(withoutImage, withoutImage.audioTrack.clips[0]!.id);
    expect(silent.assets.map(({ nodeId }) => nodeId)).toEqual(["primary"]);
    const bare = pruneAssets({ ...silent, videoTrack: { clips: [] } });
    expect(bare.assets.map(({ nodeId }) => nodeId)).toEqual(["primary"]);
  });

  it("resolves the asset backing a clip so the preview and the export agree", () => {
    const project = insertMaterial(source(), image, 4_000);
    expect(assetOf(project, "image")).toMatchObject({ width: 720, height: 1280 });
    expect(assetOf(project, "missing")).toBeUndefined();
  });
});
