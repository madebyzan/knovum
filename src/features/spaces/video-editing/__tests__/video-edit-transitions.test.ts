import { describe, expect, it } from "vitest";

import { videoEditProjectSchema } from "../../model/nodes/schema/video-edit";
import { removeClip, splitClip } from "../edits/arrange";
import { insertMaterial } from "../edits/material";
import { applyTransition, maxTransitionMs, removeTransition, resizeTransition } from "../edits/transition";
import { createVideoEditProject, type VideoAsset } from "../project";
import { clipAudioGain, compositionAt, projectDurationMs } from "../timing";

const second: VideoAsset = {
  nodeId: "second",
  sourceRevision: "second-revision",
  kind: "video",
  durationMs: 3_000,
  width: 1280,
  height: 720,
};

function twoClips() {
  const project = createVideoEditProject({
    nodeId: "source",
    sourceRevision: "revision",
    durationMs: 4_000,
    sourceWidth: 1280,
    sourceHeight: 720,
  });
  return insertMaterial(project, second, 4_000);
}

function withDissolve(durationMs = 1_000) {
  const initial = twoClips();
  return applyTransition(initial, initial.videoTrack.clips[0]!.id, "dissolve", durationMs);
}

describe("video transitions", () => {
  it("overlaps the pair by exactly the transition length and shortens the project", () => {
    const project = withDissolve();
    expect(project.transitions).toMatchObject([{ kind: "dissolve", durationMs: 1_000 }]);
    expect(project.videoTrack.clips[1]!.timelineStartMs).toBe(3_000);
    expect(projectDurationMs(project)).toBe(6_000);
    expect(videoEditProjectSchema.safeParse(project).success).toBe(true);
  });

  it("samples both clips with complementary weights inside the overlap", () => {
    const project = withDissolve();
    const composition = compositionAt(project, 3_500);
    expect(composition.layers.map(({ clip, offsetMs }) => [clip.assetId, offsetMs])).toEqual([
      ["source", 3_500],
      ["second", 500],
    ]);
    expect(composition.transition?.progress).toBe(0.5);
    expect(compositionAt(project, 4_000).layers.map(({ clip }) => clip.assetId)).toEqual(["second"]);
  });

  it("crossfades the audio of the overlapping clips the way the export does", () => {
    const project = withDissolve();
    const [outgoing, incoming] = project.videoTrack.clips;
    expect(clipAudioGain(project, outgoing!, 3_500)).toBeCloseTo(0.5, 5);
    expect(clipAudioGain(project, incoming!, 500)).toBeCloseTo(0.5, 5);
  });

  it("caps a transition at the media both neighbours can spare", () => {
    const initial = twoClips();
    const clipId = initial.videoTrack.clips[0]!.id;
    expect(maxTransitionMs(initial, clipId)).toBe(2_900);
    const project = applyTransition(initial, clipId, "wipeLeft", 10_000);
    expect(project.transitions[0]!.durationMs).toBe(2_900);
    expect(maxTransitionMs(project, project.videoTrack.clips[1]!.id)).toBe(0);
  });

  it("rejects an overlap that would consume a whole clip", () => {
    const forged = structuredClone(twoClips());
    forged.transitions = [
      {
        id: "transition",
        fromClipId: forged.videoTrack.clips[0]!.id,
        kind: "dissolve",
        durationMs: 4_000,
      },
    ];
    expect(videoEditProjectSchema.safeParse(forged).success).toBe(false);
  });

  it("resizes and removes a transition, restoring the spacing it consumed", () => {
    const project = withDissolve();
    const shorter = resizeTransition(project, project.transitions[0]!.id, 400);
    expect(shorter.videoTrack.clips[1]!.timelineStartMs).toBe(3_600);
    const removed = removeTransition(shorter, shorter.transitions[0]!.id);
    expect(removed.transitions).toEqual([]);
    expect(removed.videoTrack.clips[1]!.timelineStartMs).toBe(4_000);
    expect(projectDurationMs(removed)).toBe(7_000);
  });

  it("drops a transition whose cut no longer exists", () => {
    const project = withDissolve();
    const removed = removeClip(project, project.videoTrack.clips[1]!.id);
    expect(removed.transitions).toEqual([]);
  });

  it("moves a transition to the trailing half when its clip is split", () => {
    const project = withDissolve();
    const split = splitClip(project, 1_500);
    expect(split.transitions[0]!.fromClipId).toBe(split.videoTrack.clips[1]!.id);
    expect(projectDurationMs(split)).toBe(6_000);
  });

  it("refuses a transition where the cut cannot spare any media", () => {
    const tight = splitClip(twoClips(), 150);
    const clipId = tight.videoTrack.clips[0]!.id;
    expect(maxTransitionMs(tight, clipId)).toBe(50);
    expect(applyTransition(tight, clipId, "dissolve", 100)).toBe(tight);
  });
});
