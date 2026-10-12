import { describe, expect, it } from "vitest";

import { videoEditProjectSchema } from "../../model/nodes/schema/video-edit";
import { removeClip, splitClip } from "../clip-edits";
import { addTextOverlay, createVideoEditProject } from "../project";
import { locateTimelineTime, projectDurationMs } from "../timing";

describe("video edit project", () => {
  it("splits clips on the project timeline without changing total duration", () => {
    const source = createVideoEditProject({
      nodeId: "source",
      sourceRevision: "sha256:source",
      durationMs: 10_000,
      sourceWidth: 1080,
      sourceHeight: 1920,
    });
    const split = splitClip(source, 4_000);
    expect(split.videoTrack.clips.map(({ startMs, endMs }) => [startMs, endMs])).toEqual([
      [0, 4_000],
      [4_000, 10_000],
    ]);
    expect(projectDurationMs(split)).toBe(10_000);
  });

  it("maps a timeline position back to its source clip", () => {
    const project = splitClip(
      createVideoEditProject({
        nodeId: "source",
        sourceRevision: "sha256:source",
        durationMs: 10_000,
        sourceWidth: 1080,
        sourceHeight: 1920,
      }),
      4_000,
    );
    const trimmed = {
      ...project,
      videoTrack: {
        clips: project.videoTrack.clips.map((clip, index) =>
          index ? Object.assign({}, clip, { startMs: 6_000 }) : clip,
        ),
      },
    };
    expect(locateTimelineTime(trimmed, 4_500)?.clip.startMs).toBe(6_000);
    expect(locateTimelineTime(trimmed, 4_500)?.offsetMs).toBe(500);
  });

  it("keeps one clip and allows text in the final 100 ms", () => {
    const project = createVideoEditProject({
      nodeId: "source",
      sourceRevision: "sha256:source",
      durationMs: 2_000,
      sourceWidth: 1080,
      sourceHeight: 1920,
    });
    expect(removeClip(project, project.videoTrack.clips[0]!.id)).toBe(project);
    const withText = addTextOverlay(project, 1_900, "PingFang SC", "Text");
    expect(withText.textTrack.items[0]).toMatchObject({ startMs: 1_900, endMs: 2_000 });
  });
});

it("preserves subtitle whitespace across the persistence boundary without changing the render snapshot", () => {
  const project = createVideoEditProject({
    nodeId: "source",
    sourceRevision: "revision",
    durationMs: 2000,
    sourceWidth: 1280,
    sourceHeight: 720,
  });
  const text = "  Opening title\nSecond line  ";
  const edited = addTextOverlay(project, 0, "Arial", text);
  expect(videoEditProjectSchema.parse(edited).textTrack.items[0]!.text).toBe(text);
});
