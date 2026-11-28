import { describe, expect, it } from "vitest";

import { transcriptFixture } from "../../__tests__/support/media-transcript-fixture";
import { createSpaceNode } from "../../canvas/node";
import { formatSubtitleSrt, formatSubtitleVtt, parseSubtitleCues } from "../../media/subtitle-codec";
import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import type { SpaceVideoPayload } from "../../model/nodes/node-payload";
import { estimateSpokenDurationMs } from "../../speech/spoken-duration";
import { splitClip } from "../edits/arrange";
import { splitAudioClip } from "../edits/audio";
import {
  applyTextStyleToAll,
  defaultCaptionStyle,
  generateCaptionsFromTranscripts,
  importSubtitlesIntoProject,
  mergeTextOverlayWithNext,
  splitCaptionText,
  splitTextOverlayAt,
} from "../edits/captions";
import { commitProject } from "../edits/commit";
import { canSplitTarget, splitTargetAt } from "../edits/split-selection";
import { updateTextOverlay, updateTextStyle } from "../edits/text";
import type { VideoMaterialNode } from "../material";
import { createVideoEditProject, type VideoEditProject } from "../project";

function makeProject(): VideoEditProject {
  return createVideoEditProject({
    nodeId: "video-1",
    sourceRevision: "rev-1",
    durationMs: 12_000,
    sourceWidth: 1920,
    sourceHeight: 1080,
    frameRate: 30,
  });
}

describe("subtitle codec & caption studio bridge", () => {
  it("parses SRT and WebVTT and formats both cleanly", () => {
    const srt =
      "1\n00:00:01,000 --> 00:00:03,500\nHello world\n\n2\n00:00:04,000 --> 00:00:06,250\nSecond line\n";
    const cues = parseSubtitleCues(srt);
    expect(cues).toHaveLength(2);
    expect(cues[0]).toMatchObject({ startMs: 1_000, endMs: 3_500, text: "Hello world" });
    expect(formatSubtitleSrt(cues)).toContain("00:00:01,000 --> 00:00:03,500\nHello world");
    expect(formatSubtitleVtt(cues)).toMatch(/^WEBVTT\n\n00:00:01\.000 --> 00:00:03\.500/u);
  });

  it("splits long CJK and Latin transcript segments into readable caption lines", () => {
    const cjk = splitCaptionText(
      "这是一段非常长的中文口播内容，用来测试系统是否能够按照自然标点拆分成清晰易读的字幕段落。",
    );
    expect(cjk.length).toBeGreaterThanOrEqual(2);
    expect(cjk.every((line) => [...line].length <= 24)).toBe(true);

    const latin = splitCaptionText(
      "This is a long spoken sentence that should break cleanly on spaces so captions never overflow the video frame.",
    );
    expect(latin.length).toBeGreaterThanOrEqual(2);
    expect(latin.every((line) => line.length <= 48)).toBe(true);
  });

  it("generates timeline-accurate captions from trimmed video clips' transcripts", () => {
    const base = makeProject();
    // Trim clip to [2000ms..8000ms] starting at timeline 0ms
    const trimmed = commitProject({
      ...base,
      videoTrack: {
        clips: [{ ...base.videoTrack.clips[0]!, sourceInMs: 2_000, sourceOutMs: 8_000, timelineStartMs: 0 }],
      },
    });
    const videoNode = createSpaceNode(
      {
        kind: "video",
        title: "Interview",
        fileName: "interview.mp4",
        assetPath: "/interview.mp4",
        sourceRevision: "rev-1",
        mediaKnowledge: {
          schemaVersion: 1,
          sourceRevision: "rev-1",
          transcript: transcriptFixture("rev-1", [
            { startMs: 0, endMs: 1_500, text: "Trimmed away intro" },
            { startMs: 3_000, endMs: 5_000, text: "Kept inside clip" },
            { startMs: 9_000, endMs: 11_000, text: "Trimmed away outro" },
          ]),
        },
      },
      { x: 0, y: 0 },
      "video-1",
    ) as SpaceCanvasNode<SpaceVideoPayload>;

    const materials = new Map<string, VideoMaterialNode>([["video-1", videoNode as VideoMaterialNode]]);
    const { project, count } = generateCaptionsFromTranscripts(
      trimmed,
      materials,
      defaultCaptionStyle("Inter"),
    );
    expect(count).toBe(1);
    // Source 3000..5000 shifted by -2000 -> timeline 1000..3000
    expect(project.textTrack.items[0]).toMatchObject({
      startMs: 1_000,
      endMs: 3_000,
      text: "Kept inside clip",
      y: 0.88,
    });
  });

  it("supports SRT import, batch style sync, caption split, and merge", () => {
    const style = defaultCaptionStyle("Inter");
    const imported = importSubtitlesIntoProject(
      makeProject(),
      [
        { startMs: 0, endMs: 4_000, text: "Hello world, welcome back." },
        { startMs: 4_500, endMs: 7_000, text: "Next caption" },
      ],
      style,
    );
    expect(imported.count).toBe(2);

    const firstId = imported.project.textTrack.items[0]!.id;
    const restyled = updateTextStyle(updateTextOverlay(imported.project, firstId, { y: 0.14 }), firstId, {
      color: "#FFCC00",
    });
    const synced = applyTextStyleToAll(restyled, firstId);
    expect(synced.textTrack.items.every((item) => item.y === 0.14 && item.style.color === "#FFCC00")).toBe(
      true,
    );

    const split = splitTextOverlayAt(synced, firstId, 2_000);
    expect(split.textTrack.items).toHaveLength(3);
    expect(split.textTrack.items[0]!.endMs).toBe(2_000);
    expect(split.textTrack.items[1]!.startMs).toBe(2_000);

    const merged = mergeTextOverlayWithNext(split, firstId);
    expect(merged.textTrack.items).toHaveLength(2);
    expect(merged.textTrack.items[0]!.endMs).toBe(4_000);
  });

  it("splits selected audio clips and dispatches multi-track splitTargetAt cleanly", () => {
    const base = makeProject();
    const audioId = crypto.randomUUID();
    const withAudio = commitProject({
      ...base,
      assets: [
        ...base.assets,
        { nodeId: "audio-1", sourceRevision: "a1", kind: "audio", durationMs: 8_000, width: 0, height: 0 },
      ],
      audioTrack: {
        clips: [
          {
            id: audioId,
            assetId: "audio-1",
            sourceInMs: 0,
            sourceOutMs: 6_000,
            timelineStartMs: 1_000,
            lane: 0,
            volume: 1,
            fadeInMs: 500,
            fadeOutMs: 500,
          },
        ],
      },
    });
    expect(canSplitTarget(withAudio, { kind: "audio", id: audioId }, 4_000)).toBe(true);
    const audioResult = splitTargetAt(withAudio, { kind: "audio", id: audioId }, 4_000);
    expect(audioResult.project.audioTrack.clips).toHaveLength(2);
    expect(audioResult.nextSelection?.kind).toBe("audio");

    const directSplit = splitAudioClip(withAudio, audioId, 4_000);
    expect(directSplit.audioTrack.clips[0]!.sourceOutMs).toBe(3_000);
    expect(directSplit.audioTrack.clips[1]!.timelineStartMs).toBe(4_000);

    // Fallback splits video clip when no audio/text contains playhead
    const videoSplit = splitClip(base, 5_000);
    expect(videoSplit.videoTrack.clips).toHaveLength(2);
  });

  it("estimates realistic spoken durations for CJK and English scripts", () => {
    expect(estimateSpokenDurationMs("")).toBe(0);
    expect(estimateSpokenDurationMs("一".repeat(220))).toBe(60_000);
    expect(estimateSpokenDurationMs("word ".repeat(150))).toBe(60_000);
  });
});
