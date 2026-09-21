import { describe, expect, it } from "vitest";

import {
  editMediaChapter,
  editMediaSummary,
  editTranscriptSegment,
  mediaKnowledgeIsStale,
  shiftMediaChapterStart,
} from "../editing";
import type { SpaceMediaKnowledge } from "../schema";

function knowledge(): SpaceMediaKnowledge {
  return {
    transcript: {
      language: "en",
      durationMs: 9_000,
      text: "one two three",
      segments: [
        { startMs: 0, endMs: 3_000, text: "one" },
        { startMs: 3_000, endMs: 6_000, text: "two" },
        { startMs: 6_000, endMs: 9_000, text: "three" },
      ],
      subtitleCues: [
        { startMs: 0, endMs: 3_000, text: "one" },
        { startMs: 3_000, endMs: 6_000, text: "two" },
        { startMs: 6_000, endMs: 9_000, text: "three" },
      ],
    },
    sourceRevision: "sha256:source",
    schemaVersion: 4,
    insights: {
      summary: "Original",
      chapters: [
        { startMs: 0, endMs: 3_000, title: "One", summary: "First" },
        { startMs: 3_000, endMs: 9_000, title: "Two", summary: "Second" },
      ],
      promptVersion: 1,
      generatedAt: 1,
      provider: "provider",
      model: "model",
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

  it("rejects empty chapter text", () => {
    expect(() => editMediaChapter(knowledge(), 0, { title: "", summary: "Valid" })).toThrow(
      "Chapter title cannot be empty",
    );
  });
});
