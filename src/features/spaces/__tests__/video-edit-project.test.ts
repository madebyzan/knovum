import { describe, expect, it } from "vitest";

import {
  addTextOverlay,
  createVideoEditProject,
  locateTimelineTime,
  projectDuration,
  removeClip,
  splitClip,
} from "../video-editing/project";

describe("video edit project", () => {
  it("splits clips on the project timeline without changing total duration", () => {
    const source = createVideoEditProject("sha256:source", 10_000, 1080, 1920);
    const split = splitClip(source, 4_000);
    expect(split.videoTrack.clips.map(({ startMs, endMs }) => [startMs, endMs])).toEqual([
      [0, 4_000],
      [4_000, 10_000],
    ]);
    expect(projectDuration(split)).toBe(10_000);
  });

  it("maps a timeline position back to its source clip", () => {
    const project = splitClip(createVideoEditProject("sha256:source", 10_000, 1080, 1920), 4_000);
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

  it("keeps one video clip and bounds new timed text", () => {
    const project = createVideoEditProject("sha256:source", 2_000, 1080, 1920);
    expect(removeClip(project, project.videoTrack.clips[0]!.id)).toBe(project);
    const withText = addTextOverlay(project, 1_900, "PingFang SC");
    expect(withText.textTrack.items[0]).toMatchObject({ startMs: 1_500, endMs: 2_000 });
  });
});
