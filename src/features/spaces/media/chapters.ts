export function normalizeChapterEnds<T extends { startMs: number }>(
  chapters: readonly T[],
  durationMs: number,
): Array<T & { endMs: number }> {
  return chapters.map((chapter, index) =>
    Object.assign({}, chapter, { endMs: chapters[index + 1]?.startMs ?? durationMs }),
  );
}
