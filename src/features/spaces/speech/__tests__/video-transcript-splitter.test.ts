import { describe, expect, it } from "vitest";

import type { SpaceMediaTranscript, SpaceTranscriptSegment } from "../../media/schema/transcript";
import { DIALOGUE_PAUSE_SPLIT_THRESHOLD_MS, linesFromTranscript } from "../video-transcript-splitter";

function makeUnit(params: {
  text: string;
  startChar: number;
  endChar: number;
  startMs: number;
  endMs: number;
}) {
  return {
    id: "1".repeat(64),
    text: params.text,
    startCharacter: params.startChar,
    endCharacter: params.endChar,
    startMs: params.startMs,
    endMs: params.endMs,
    score: 0.95,
    unalignedReason: null,
    unitKind: "word" as const,
  };
}

function makeSegment(params: {
  id: string;
  text: string;
  startMs: number;
  endMs: number;
  units?: SpaceTranscriptSegment["units"];
  speakerId?: string | null;
}): SpaceTranscriptSegment {
  return {
    id: params.id,
    revision: 2,
    startMs: params.startMs,
    endMs: params.endMs,
    startSample: params.startMs * 16,
    endSample: params.endMs * 16,
    speakerId: params.speakerId !== undefined ? params.speakerId : "host",
    text: params.text,
    units: params.units ?? [],
  };
}

function makeTranscript(segments: SpaceTranscriptSegment[]): SpaceMediaTranscript {
  return {
    id: "a".repeat(64),
    sourceRevision: "rev-1",
    revision: 1,
    language: "zh-CN",
    durationMs: 10000,
    text: segments.map((s) => s.text).join(" "),
    subtitleCues: [],
    speakerTracks: [],
    provenance: { engineId: "whisper", modelSpecId: "base", modelRevision: "1", timing: "engine_words" },
    segments,
  };
}

describe("video-transcript-splitter", () => {
  it("defines the standard dialogue pause threshold as 1200ms", () => {
    expect(DIALOGUE_PAUSE_SPLIT_THRESHOLD_MS).toBe(1200);
  });

  it("keeps a segment as a single line when units are missing or unaligned", () => {
    const seg = makeSegment({
      id: "b".repeat(64),
      text: "大家好，这是一段没有词级时间戳的音频。",
      startMs: 500,
      endMs: 3000,
      speakerId: "speaker-1",
    });
    const result = linesFromTranscript(makeTranscript([seg]), "voice-1");
    expect(result.lines).toHaveLength(1);
    expect(result.lines[0]).toMatchObject({
      text: seg.text,
      speakerKey: "speaker-1",
      voiceRevisionId: "voice-1",
      startSeconds: "0.5",
      endSeconds: "3",
      sourceSegmentId: seg.id,
      sourceSegmentRevision: 2,
    });
    expect(result.intervals).toEqual([{ startMs: 500, endMs: 3000 }]);
  });

  it("does not split when word pauses are below the threshold", () => {
    const units = [
      makeUnit({ text: "你好", startChar: 0, endChar: 2, startMs: 1000, endMs: 1500 }),
      makeUnit({ text: "世界", startChar: 3, endChar: 5, startMs: 1700, endMs: 2500 }),
    ];
    const seg = makeSegment({
      id: "c".repeat(64),
      text: "你好 世界",
      startMs: 1000,
      endMs: 2500,
      units,
      speakerId: "narrator",
    });
    const result = linesFromTranscript(makeTranscript([seg]), "voice-default");
    expect(result.lines).toHaveLength(1);
    expect(result.lines[0]?.text).toBe("你好 世界");
    expect(result.lines[0]?.startSeconds).toBe("1");
    expect(result.lines[0]?.endSeconds).toBe("2.5");
    expect(result.intervals).toEqual([{ startMs: 1000, endMs: 2500 }]);
  });

  it("splits dialogue when pause between words is >= 1200ms", () => {
    const units = [
      makeUnit({ text: "大家好今天介绍产品", startChar: 0, endChar: 9, startMs: 1000, endMs: 2500 }),
      makeUnit({ text: "我们来看外观", startChar: 11, endChar: 17, startMs: 4500, endMs: 6000 }), // 2000ms gap
    ];
    const seg = makeSegment({
      id: "d".repeat(64),
      text: "大家好今天介绍产品， 我们来看外观。",
      startMs: 1000,
      endMs: 6000,
      units,
    });
    const result = linesFromTranscript(makeTranscript([seg]), "voice-pro");
    expect(result.lines).toHaveLength(2);
    expect(result.lines[0]).toMatchObject({
      text: "大家好今天介绍产品，",
      speakerKey: "host",
      voiceRevisionId: "voice-pro",
      startSeconds: "1",
      endSeconds: "2.5",
      sourceSegmentId: `${seg.id}:0`,
      sourceSegmentRevision: 2,
    });
    expect(result.lines[1]).toMatchObject({
      text: "我们来看外观。",
      speakerKey: "host",
      voiceRevisionId: "voice-pro",
      startSeconds: "4.5",
      endSeconds: "6",
      sourceSegmentId: `${seg.id}:1`,
      sourceSegmentRevision: 2,
    });
    expect(result.intervals).toEqual([
      { startMs: 1000, endMs: 2500 },
      { startMs: 4500, endMs: 6000 },
    ]);
  });

  it("handles custom threshold and multiple segments", () => {
    const units = [
      makeUnit({ text: "Part one", startChar: 0, endChar: 8, startMs: 0, endMs: 1000 }),
      makeUnit({ text: "Part two", startChar: 10, endChar: 18, startMs: 2000, endMs: 3000 }),
    ];
    const seg = makeSegment({
      id: "e".repeat(64),
      text: "Part one. Part two.",
      startMs: 0,
      endMs: 3000,
      units,
      speakerId: null,
    });
    const transcript = makeTranscript([seg]);

    expect(linesFromTranscript(transcript, "v1").lines).toHaveLength(1);
    const split = linesFromTranscript(transcript, "v1", 800);
    expect(split.lines).toHaveLength(2);
    expect(split.lines[0]?.text).toBe("Part one.");
    expect(split.lines[1]?.text).toBe("Part two.");
  });
});
