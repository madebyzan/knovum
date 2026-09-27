import pLimit from "p-limit";
import { z } from "zod";

import { completeGeneralWithMetadata } from "../../../foundation/ai/ai-runtime";
import { parseStructuredJson } from "../../../foundation/ai/structured-output";
import { finalizeMediaChapters } from "./chapters";
import type { SpaceMediaKnowledge, SpaceMediaTranscript } from "./knowledge-schema";
import { mediaCandidateText, mediaKnowledgeInstruction, mediaTranscriptChunks } from "./prompts";

const chunkInference = pLimit(3);

const aiKnowledgeSchema = z.object({
  summary: z.string().trim().min(1).max(16_000),
  chapters: z
    .array(
      z.object({
        startMs: z.number().int().nonnegative(),
        title: z.string().trim().min(1).max(180),
        summary: z.string().trim().min(1).max(2_000),
      }),
    )
    .min(1)
    .max(2_000),
});

export function createLocalMediaKnowledge(
  transcript: SpaceMediaTranscript,
  sourceRevision: string,
): SpaceMediaKnowledge {
  return { transcript, sourceRevision, schemaVersion: 4 };
}

export async function generateSpaceMediaInsights(
  knowledge: SpaceMediaKnowledge,
): Promise<SpaceMediaKnowledge> {
  const chunks = mediaTranscriptChunks(knowledge.transcript);
  const completions = await Promise.all(
    chunks.map((chunk) =>
      chunkInference(() =>
        completeGeneralWithMetadata({
          instruction: mediaKnowledgeInstruction("chunk"),
          text: chunk,
        }),
      ),
    ),
  );
  const candidates = completions.map((completion) => parseStructuredJson(completion.text, aiKnowledgeSchema));
  let provider = completions[0]?.provider ?? "";
  let model = completions[0]?.model ?? "";
  let insights = candidates[0]!;
  if (candidates.length > 1) {
    const completion = await completeGeneralWithMetadata({
      instruction: mediaKnowledgeInstruction("final"),
      text: mediaCandidateText(candidates),
    });
    insights = parseStructuredJson(completion.text, aiKnowledgeSchema);
    provider = completion.provider;
    model = completion.model;
  }
  return {
    ...knowledge,
    insights: {
      summary: insights.summary,
      chapters: finalizeMediaChapters(insights.chapters, knowledge.transcript),
      promptVersion: 1,
      generatedAt: Date.now(),
      provider,
      model,
    },
  };
}
