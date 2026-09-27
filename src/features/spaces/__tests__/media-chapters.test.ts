import { describe, expect, it } from "vitest";

import { normalizeChapterEnds } from "../media/chapters";

describe("manual chapter editing", () => {
  it("keeps chapter content and derives contiguous end timestamps", () => {
    const chapters = normalizeChapterEnds(
      [
        { startMs: 0, title: "Intro" },
        { startMs: 8000, title: "Topic" },
        { startMs: 20000, title: "Closing" },
      ],
      30000,
    );
    expect(chapters).toEqual([
      { startMs: 0, endMs: 8000, title: "Intro" },
      { startMs: 8000, endMs: 20000, title: "Topic" },
      { startMs: 20000, endMs: 30000, title: "Closing" },
    ]);
  });
});
