import { describe, expect, it } from "vitest";

import { videoEditProjectSchema } from "../../model/nodes/schema/video-edit";
import { commitProject } from "../edits/commit";
import { addTextOverlay } from "../edits/text";
import { createVideoEditProject, defaultTextStyle } from "../project";
import { projectDurationMs } from "../timing";

function source() {
  return createVideoEditProject({
    nodeId: "source",
    sourceRevision: "revision",
    durationMs: 10_000,
    sourceWidth: 1081,
    sourceHeight: 1920,
  });
}

function measured(frameRate: 24 | 25 | 30 | 60) {
  return createVideoEditProject({
    nodeId: "source",
    sourceRevision: "revision",
    durationMs: 10_000,
    sourceWidth: 3840,
    sourceHeight: 2160,
    frameRate,
  });
}

describe("video edit project contract", () => {
  it("places the owning source at the timeline origin and derives an even output frame", () => {
    const project = source();
    expect(project.version).toBe(2);
    expect(project.output).toMatchObject({ width: 1080, height: 1920, frameRate: 30 });
    expect(project.videoTrack.clips).toMatchObject([
      { assetId: "source", sourceInMs: 0, sourceOutMs: 10_000, timelineStartMs: 0 },
    ]);
    expect(projectDurationMs(project)).toBe(10_000);
    expect(videoEditProjectSchema.safeParse(project).success).toBe(true);
  });

  it("starts the output at the resolution and cadence of the source it was measured from", () => {
    expect(measured(60).output).toMatchObject({ width: 3840, height: 2160, frameRate: 60 });
    expect(measured(24).output.frameRate).toBe(24);
    expect(source().output.frameRate).toBe(30);
  });

  it("rejects a project stored under the previous contract instead of migrating it", () => {
    const legacy = {
      version: 1,
      sourceRevision: "revision",
      sourceWidth: 1280,
      sourceHeight: 720,
      videoTrack: { clips: [{ id: "clip", assetId: "source", startMs: 0, endMs: 1_000 }] },
      textTrack: { items: [] },
      transform: { rotation: 0, flipHorizontal: false, flipVertical: false },
    };
    expect(videoEditProjectSchema.safeParse(legacy).success).toBe(false);
  });

  it("rejects clips, audio and transitions that reference media the project does not own", () => {
    const forged = structuredClone(source());
    forged.videoTrack.clips[0]!.assetId = "unknown";
    expect(videoEditProjectSchema.safeParse(forged).success).toBe(false);
    const orphaned = structuredClone(source());
    orphaned.transitions = [{ id: "transition", fromClipId: "missing", kind: "dissolve", durationMs: 500 }];
    expect(videoEditProjectSchema.safeParse(orphaned).success).toBe(false);
  });

  it("clamps overlay text to the timeline and drops what no longer fits", () => {
    const style = defaultTextStyle("Arial");
    const project = addTextOverlay(source(), 9_500, style, "Closing");
    expect(project.textTrack.items[0]).toMatchObject({ startMs: 9_500, endMs: 10_000 });
    const shortened = structuredClone(project);
    shortened.videoTrack.clips[0]!.sourceOutMs = 9_550;
    expect(commitProject(shortened).textTrack.items).toEqual([]);
  });

  it("preserves overlay whitespace across the persistence boundary", () => {
    const text = "  Opening title\nSecond line  ";
    const project = addTextOverlay(source(), 0, defaultTextStyle("Arial"), text);
    expect(videoEditProjectSchema.parse(project).textTrack.items[0]!.text).toBe(text);
  });

  it("keeps the last clip so a project always has media to show", () => {
    const project = source();
    const commited = commitProject(project);
    expect(commited.videoTrack.clips).toHaveLength(1);
    expect(commited.assets.map(({ nodeId }) => nodeId)).toEqual(["source"]);
  });
});
