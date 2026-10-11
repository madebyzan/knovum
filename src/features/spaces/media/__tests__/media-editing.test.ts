import { describe, expect, it } from "vitest";

import { transcriptFixture } from "../../__tests__/support/media-transcript-fixture";
import {
  editMediaChapter,
  editMediaSummary,
  editTranscriptSegment,
  mediaKnowledgeIsStale,
  shiftMediaChapterStart,
} from "../knowledge-editing";
import type { SpaceMediaKnowledge } from "../schema/knowledge";

function knowledge(): SpaceMediaKnowledge {
  return {
    transcript: transcriptFixture("sha256:source", [
      { startMs: 0, endMs: 3000, text: "one" },
      { startMs: 3000, endMs: 6000, text: "two" },
      { startMs: 6000, endMs: 9000, text: "three" },
    ]),
    sourceRevision: "sha256:source",
    schemaVersion: 1,
    insights: {
      summary: "Original",
      chapters: [
        { startMs: 0, endMs: 3_000, title: "One", summary: "First" },
        { startMs: 3_000, endMs: 9_000, title: "Two", summary: "Second" },
      ],
      promptVersion: 1,
      generatedAt: 1,
      executionSource: { type: "model", provider: "provider", model: "model" },
    },
  };
}

describe("Space media editing", () => {
  it("marks human edits without changing the generated source revision", () => {
    const updated = editMediaSummary(knowledge(), "  Updated summary  ");
    expect(updated.insights!.summary).toBe("Updated summary");
    expect(updated.insights!.editedAt).toBeTypeOf("number");
    expect(updated.sourceRevision).toBe("sha256:source");
  });

  it("moves chapter boundaries only along transcript starts and keeps contiguous ends", () => {
    const updated = shiftMediaChapterStart(knowledge(), 1, 1);
    expect(updated.insights!.chapters).toEqual([
      { startMs: 0, endMs: 6_000, title: "One", summary: "First" },
      { startMs: 6_000, endMs: 9_000, title: "Two", summary: "Second" },
    ]);
  });

  it("edits a transcript segment and refreshes the combined text", () => {
    const updated = editTranscriptSegment(knowledge(), 1, "two edited");
    expect(updated.transcript.segments[1]!.text).toBe("two edited");
    expect(updated.transcript.text).toBe("one two edited three");
  });

  it("detects analysis staleness from the source revision", () => {
    expect(mediaKnowledgeIsStale(knowledge(), "sha256:source")).toBe(false);
    expect(mediaKnowledgeIsStale(knowledge(), "sha256:changed")).toBe(true);
  });

  it("keeps neighboring subtitles when an edited segment shares a cue with them", () => {
    const original = knowledge();
    original.transcript.subtitleCues = [
      { startMs: 0, endMs: 6_000, text: "one two" },
      { startMs: 6_000, endMs: 9_000, text: "three" },
    ];
    const updated = editTranscriptSegment(original, 0, "one edited");
    expect(updated.transcript.subtitleCues).toEqual([
      { startMs: 0, endMs: 3_000, text: "one edited" },
      { startMs: 3_000, endMs: 6_000, text: "two" },
      { startMs: 6_000, endMs: 9_000, text: "three" },
    ]);
    expect(updated.transcript.segments[1]).toEqual(original.transcript.segments[1]);
  });

  it("invalidates only edited timing while preserving artifact identity and other alignment", () => {
    const original = knowledge();
    original.transcript.alignment = {
      engineId: "qwen3",
      modelSpecId: "forced-aligner",
      modelRevision: "fixed",
      alignedAt: 1,
    };
    const unchanged = structuredClone(original.transcript.segments[0]);
    const updated = editTranscriptSegment(original, 1, "two edited");
    expect(updated.transcript.id).toBe(original.transcript.id);
    expect(updated.transcript.sourceRevision).toBe(original.transcript.sourceRevision);
    expect(updated.transcript.revision).toBe(original.transcript.revision + 1);
    expect(updated.transcript.segments[1]!.revision).toBe(2);
    expect(updated.transcript.segments[1]!.units[0]).toMatchObject({
      text: "two edited",
      startMs: null,
      endMs: null,
      unitKind: "segment",
    });
    expect(updated.transcript.segments[0]).toEqual(unchanged);
    expect(updated.transcript.alignment).toEqual(original.transcript.alignment);
    expect(updated.insights).toBeUndefined();
    expect(original.transcript.segments[1]!.text).toBe("two");
  });

  it("rejects empty chapter text", () => {
    expect(() => editMediaChapter(knowledge(), 0, { title: "", summary: "Valid" })).toThrow(
      "Chapter title cannot be empty",
    );
  });
});
