import { describe, expect, it } from "vitest";

import type { SpaceMediaTranscript } from "../../media/schema/transcript";
import { speakerReferences } from "../speaker-references";

function transcript(segments: Array<[speakerId: string | null, startMs: number, endMs: number]>) {
  return {
    segments: segments.map(([speakerId, startMs, endMs]) => ({ speakerId, startMs, endMs })),
  } as unknown as SpaceMediaTranscript;
}

describe("speakerReferences", () => {
  it("selects each speaker's longest uninterrupted turn, in the order they first speak", () => {
    const references = speakerReferences(
      transcript([
        ["B", 0, 6_000],
        ["A", 6_500, 9_000],
        ["A", 9_500, 14_000],
        ["B", 15_000, 18_000],
        ["A", 30_000, 36_000],
      ]),
      15_000,
    );
    expect(references).toEqual([
      { speakerId: "B", startMs: 0, endMs: 6_000 },
      { speakerId: "A", startMs: 6_500, endMs: 14_000 },
    ]);
  });

  it("stops at the last segment boundary within the model's limit instead of cutting a word", () => {
    const references = speakerReferences(
      transcript([
        ["A", 0, 7_000],
        ["A", 7_200, 13_000],
        ["A", 13_100, 20_000],
      ]),
      15_000,
    );
    expect(references).toEqual([{ speakerId: "A", startMs: 0, endMs: 13_000 }]);
  });

  it("caps a single segment longer than the limit at the limit", () => {
    expect(speakerReferences(transcript([["A", 1_000, 40_000]]), 15_000)).toEqual([
      { speakerId: "A", startMs: 1_000, endMs: 16_000 },
    ]);
  });

  it("splits a turn at another speaker or a long pause and skips speakers too brief to clone", () => {
    const references = speakerReferences(
      transcript([
        ["A", 0, 3_000],
        ["A", 5_000, 8_000],
        ["B", 8_100, 10_000],
        [null, 10_000, 20_000],
      ]),
      15_000,
    );
    expect(references).toEqual([]);
  });
});
