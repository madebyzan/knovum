import { z } from "zod";

import { executionSourceSchema } from "../../../../foundation/ai/schema/models";
import { mediaSourceRevisionSchema, spaceMediaTranscriptSchema } from "./transcript";
import { validateMediaTranscript } from "./transcript-validation";

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
    executionSource: executionSourceSchema,
  })
  .strict();

export const spaceMediaKnowledgeSchema = z
  .object({
    transcript: spaceMediaTranscriptSchema.superRefine(validateMediaTranscript),
    sourceRevision: mediaSourceRevisionSchema,
    schemaVersion: z.literal(5),
    insights: insightsSchema.optional(),
  })
  .strict()
  .superRefine((knowledge, context) => {
    const { durationMs, segments } = knowledge.transcript;
    const transcriptStarts = new Set(segments.map((segment) => segment.startMs));
    if (knowledge.sourceRevision !== knowledge.transcript.sourceRevision) {
      context.addIssue({
        code: "custom",
        path: ["transcript", "sourceRevision"],
        message: "Transcript source revision must match media knowledge",
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

export interface SpaceTranscriptEdit {
  index: number;
  text: string;
}
export type SpaceMediaChapter = z.infer<typeof spaceMediaChapterSchema>;
export type SpaceMediaInsights = z.infer<typeof insightsSchema>;
export type SpaceMediaKnowledge = z.infer<typeof spaceMediaKnowledgeSchema>;
