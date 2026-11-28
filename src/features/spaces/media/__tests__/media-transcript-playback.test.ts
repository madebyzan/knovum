import { describe, expect, it, vi } from "vitest";

import { claimActiveMediaPlayback, releaseActiveMediaPlayback } from "../active-media-playback";
import { buildSpeakerMap } from "../speaker-display";
import { formatTranscriptPlainText, formatTranscriptSrt } from "../transcript-export";
import { buildTranscriptWordRuns } from "../transcript-word-runs";

describe("active-media-playback", () => {
  it("pauses previous active player when another claims playback", () => {
    const pauseA = vi.fn<() => void>();
    const pauseB = vi.fn<() => void>();
    claimActiveMediaPlayback(pauseA);
    claimActiveMediaPlayback(pauseB);
    expect(pauseA).toHaveBeenCalledTimes(1);
    expect(pauseB).not.toHaveBeenCalled();
    releaseActiveMediaPlayback(pauseB);
  });
});

describe("speaker-display & transcript-export", () => {
  const segments = [
    {
      id: "a".repeat(64),
      revision: 1,
      startMs: 1_500,
      endMs: 4_250,
      startSample: 0,
      endSample: 100,
      speakerId: "SPEAKER_00",
      text: "Hello everyone.",
      units: [],
    },
    {
      id: "b".repeat(64),
      revision: 1,
      startMs: 5_000,
      endMs: 8_100,
      startSample: 100,
      endSample: 200,
      speakerId: "SPEAKER_01",
      text: "Welcome back.",
      units: [],
    },
  ];
  const speakers = buildSpeakerMap(
    [
      { speakerId: "SPEAKER_00", turns: [{ startMs: 1_500, endMs: 4_250 }] },
      { speakerId: "SPEAKER_01", turns: [{ startMs: 5_000, endMs: 8_100 }] },
    ],
    (index) => `Speaker ${index}`,
  );

  it("formats diarized speaker badges cleanly", () => {
    expect(speakers.get("SPEAKER_00")?.label).toBe("Speaker 1");
    expect(speakers.get("SPEAKER_01")?.label).toBe("Speaker 2");
  });

  it("exports plain text and valid SRT timestamps with speakers", () => {
    expect(formatTranscriptPlainText(segments, speakers)).toBe(
      "[0:01] Speaker 1: Hello everyone.\n[0:05] Speaker 2: Welcome back.",
    );
    expect(formatTranscriptSrt(segments, speakers)).toContain(
      "00:00:01,500 --> 00:00:04,250\nSpeaker 1: Hello everyone.",
    );
    expect(formatTranscriptSrt(segments, speakers)).toContain(
      "2\n00:00:05,000 --> 00:00:08,100\nSpeaker 2: Welcome back.",
    );
  });

  it("preserves exact spaces and punctuation when slicing active timed word runs", () => {
    const runs = buildTranscriptWordRuns({
      id: "c".repeat(64),
      revision: 1,
      startMs: 30_000,
      endMs: 32_000,
      startSample: 0,
      endSample: 100,
      speakerId: null,
      text: "And how does that feel?",
      units: [
        {
          id: "u1",
          text: "And",
          startCharacter: 0,
          endCharacter: 3,
          startMs: 30_000,
          endMs: 30_300,
          score: 1,
          unalignedReason: null,
          unitKind: "word",
        },
        {
          id: "u2",
          text: "how",
          startCharacter: 4,
          endCharacter: 7,
          startMs: 30_300,
          endMs: 30_600,
          score: 1,
          unalignedReason: null,
          unitKind: "word",
        },
        {
          id: "u3",
          text: "does",
          startCharacter: 8,
          endCharacter: 12,
          startMs: 30_600,
          endMs: 30_900,
          score: 1,
          unalignedReason: null,
          unitKind: "word",
        },
        {
          id: "u4",
          text: "that",
          startCharacter: 13,
          endCharacter: 17,
          startMs: 30_900,
          endMs: 31_200,
          score: 1,
          unalignedReason: null,
          unitKind: "word",
        },
        {
          id: "u5",
          text: "feel?",
          startCharacter: 18,
          endCharacter: 23,
          startMs: 31_200,
          endMs: 32_000,
          score: 1,
          unalignedReason: null,
          unitKind: "word",
        },
      ],
    });
    expect(runs).not.toBeNull();
    expect(runs!.map((run) => run.text).join("")).toBe("And how does that feel?");
    expect(runs!.filter((run) => run.kind === "gap")).toHaveLength(4);
  });
});
