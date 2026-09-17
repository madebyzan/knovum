import { z } from "zod";

const spaceTranscriptSegmentSchema = z
  .object({
    startMs: z.number().int().nonnegative().max(Number.MAX_SAFE_INTEGER),
    endMs: z.number().int().nonnegative().max(Number.MAX_SAFE_INTEGER),
    text: z.string().trim().min(1).max(16_000),
  })
  .strict();

export const spaceSubtitleCueSchema = z
  .object({
    startMs: z.number().int().nonnegative().max(Number.MAX_SAFE_INTEGER),
    endMs: z.number().int().positive().max(Number.MAX_SAFE_INTEGER),
    text: z.string().trim().min(1).max(16_000),
  })
  .strict();

const spaceMediaTranscriptSchema = z
  .object({
    language: z.string().trim().min(1).max(64),
    durationMs: z.number().int().nonnegative().max(Number.MAX_SAFE_INTEGER),
    text: z
      .string()
      .trim()
      .max(4 * 1024 * 1024),
    segments: z.array(spaceTranscriptSegmentSchema).max(100_000),
    subtitleCues: z.array(spaceSubtitleCueSchema).max(100_000),
    waveform: z.array(z.number().min(0).max(1)).max(1_200),
  })
  .strict();

const spaceMediaChapterSchema = z
  .object({
    startMs: z.number().int().nonnegative().max(Number.MAX_SAFE_INTEGER),
    endMs: z.number().int().nonnegative().max(Number.MAX_SAFE_INTEGER),
    title: z.string().trim().min(1).max(180),
    summary: z.string().trim().min(1).max(2_000),
  })
  .strict();

const insightsSchema = z
  .object({
    summary: z.string().trim().min(1).max(16_000),
    chapters: z.array(spaceMediaChapterSchema).min(1).max(2_000),
    promptVersion: z.literal(1),
    generatedAt: z.number().int().nonnegative().max(Number.MAX_SAFE_INTEGER),
    editedAt: z.number().int().nonnegative().max(Number.MAX_SAFE_INTEGER).optional(),
    provider: z.string().trim().min(1).max(128),
    model: z.string().trim().min(1).max(256),
  })
  .strict();

export const spaceMediaKnowledgeSchema = z
  .object({
    transcript: spaceMediaTranscriptSchema,
    sourceRevision: z.string().trim().min(1).max(128),
    schemaVersion: z.literal(3),
    insights: insightsSchema.optional(),
  })
  .strict()
  .superRefine((knowledge, context) => {
    const { durationMs, segments } = knowledge.transcript;
    const transcriptStarts = new Set(segments.map((segment) => segment.startMs));
    const invalidSegments = segments.some(
      (segment, index) =>
        segment.endMs < segment.startMs ||
        segment.endMs > durationMs ||
        (index > 0 && segment.startMs < segments[index - 1]!.startMs),
    );
    if (invalidSegments) {
      context.addIssue({
        code: "custom",
        path: ["transcript", "segments"],
        message: "Transcript segments must be ordered and bounded",
      });
    }

    const invalidSubtitleCues = knowledge.transcript.subtitleCues.some(
      (cue, index, cues) =>
        cue.endMs <= cue.startMs ||
        cue.endMs > durationMs ||
        (index > 0 && cue.startMs < cues[index - 1]!.startMs),
    );
    if (invalidSubtitleCues) {
      context.addIssue({
        code: "custom",
        path: ["transcript", "subtitleCues"],
        message: "Subtitle cues must be ordered and bounded",
      });
    }

    const chapters = knowledge.insights?.chapters ?? [];
    const invalidChapters =
      chapters.length > 0 &&
      (chapters[0]?.startMs !== segments[0]?.startMs ||
        chapters.some(
          (chapter, index) =>
            chapter.endMs <= chapter.startMs ||
            chapter.endMs > durationMs ||
            !transcriptStarts.has(chapter.startMs) ||
            (index > 0 && chapter.startMs <= chapters[index - 1]!.startMs) ||
            chapter.endMs !== (chapters[index + 1]?.startMs ?? durationMs),
        ));
    if (invalidChapters) {
      context.addIssue({
        code: "custom",
        path: ["insights", "chapters"],
        message: "Chapters must be ordered, contiguous, and anchored to transcript segments",
      });
    }
  });

export type SpaceTranscriptSegment = z.infer<typeof spaceTranscriptSegmentSchema>;
export interface SpaceTranscriptEdit {
  index: number;
  text: string;
}
export type SpaceSubtitleCue = z.infer<typeof spaceSubtitleCueSchema>;
export type SpaceMediaTranscript = z.infer<typeof spaceMediaTranscriptSchema>;
export type SpaceMediaChapter = z.infer<typeof spaceMediaChapterSchema>;
export type SpaceMediaInsights = z.infer<typeof insightsSchema>;
export type SpaceMediaKnowledge = z.infer<typeof spaceMediaKnowledgeSchema>;
