import { describe, expect, it } from "vitest";

import { finalizeMediaChapters } from "../media/chapters";
import type { SpaceMediaTranscript } from "../media/schema";

const transcript: SpaceMediaTranscript = {
  language: "en",
  durationMs: 30_000,
  text: "Intro Topic Closing",
  waveform: [],
  segments: [
    { startMs: 0, endMs: 8_000, text: "Intro" },
    { startMs: 8_000, endMs: 20_000, text: "Topic" },
    { startMs: 20_000, endMs: 30_000, text: "Closing" },
  ],
  subtitleCues: [
    { startMs: 0, endMs: 8_000, text: "Intro" },
    { startMs: 8_000, endMs: 20_000, text: "Topic" },
    { startMs: 20_000, endMs: 30_000, text: "Closing" },
  ],
};

describe("finalizeMediaChapters", () => {
  it("sorts chapters and derives non-overlapping end timestamps", () => {
    const chapters = finalizeMediaChapters(
      [
        { startMs: 20_000, title: "Closing", summary: "Wrap up" },
        { startMs: 0, title: "Intro", summary: "Opening" },
        { startMs: 8_000, title: "Topic", summary: "Main topic" },
      ],
      transcript,
    );

    expect(chapters.map(({ startMs, endMs }) => [startMs, endMs])).toEqual([
      [0, 8_000],
      [8_000, 20_000],
      [20_000, 30_000],
    ]);
  });

  it("rejects chapter timestamps that cannot seek to a transcript boundary", () => {
    expect(() =>
      finalizeMediaChapters([{ startMs: 5_000, title: "Invalid", summary: "No segment here" }], transcript),
    ).toThrow("must reference transcript segments");
  });

  it("requires the first chapter to start with the transcript", () => {
    expect(() =>
      finalizeMediaChapters(
        [{ startMs: 8_000, title: "Topic", summary: "Starts after the intro" }],
        transcript,
      ),
    ).toThrow("must start with the transcript");
  });

  it("rejects duplicate chapter timestamps", () => {
    expect(() =>
      finalizeMediaChapters(
        [
          { startMs: 0, title: "A", summary: "A" },
          { startMs: 0, title: "B", summary: "B" },
        ],
        transcript,
      ),
    ).toThrow("must be unique");
  });
});
