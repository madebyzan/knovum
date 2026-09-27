import { sortBy, uniqBy } from "es-toolkit";

import type { SpaceMediaChapter, SpaceMediaTranscript } from "./knowledge-schema";
import type { MediaChapterCandidate } from "./prompts";

export function normalizeChapterEnds<T extends { startMs: number }>(
  chapters: readonly T[],
  durationMs: number,
): Array<T & { endMs: number }> {
  return chapters.map((chapter, index) =>
    Object.assign({}, chapter, { endMs: chapters[index + 1]?.startMs ?? durationMs }),
  );
}

export function finalizeMediaChapters(
  candidates: readonly MediaChapterCandidate[],
  transcript: SpaceMediaTranscript,
): SpaceMediaChapter[] {
  const chapters = sortBy(candidates, [(chapter) => chapter.startMs]);
  if (uniqBy(chapters, (chapter) => chapter.startMs).length !== chapters.length)
    throw new Error("Media chapter timestamps must be unique");
  const transcriptStarts = new Set(transcript.segments.map((segment) => segment.startMs));
  if (chapters.some((chapter) => !transcriptStarts.has(chapter.startMs)))
    throw new Error("Media chapter timestamps must reference transcript segments");
  if (chapters[0]?.startMs !== transcript.segments[0]?.startMs)
    throw new Error("Media chapters must start with the transcript");
  return normalizeChapterEnds(chapters, transcript.durationMs);
}
