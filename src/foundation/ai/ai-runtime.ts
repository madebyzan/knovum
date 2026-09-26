import type { AiError } from "../../shared/contracts/generated/AiError";
import type { CompletionCommandRequest } from "../../shared/contracts/generated/CompletionCommandRequest";
import type { CompletionOutput as AiCompletionResponse } from "../../shared/contracts/generated/CompletionOutput";
import { aiErrorCodes } from "../../shared/contracts/generated/contract-values";
export type { CompletionOutput as AiCompletionResponse } from "../../shared/contracts/generated/CompletionOutput";

export type { CompletionStreamEvent as AiCompletionStreamEvent } from "../../shared/contracts/generated/CompletionStreamEvent";
import { z } from "zod";

import { decodeCommandError } from "../ipc/command-error";
import { invokeCommand as invoke } from "../ipc/invoke-command";
import { aiExecutionTargetPayload, loadAiExecutionTarget } from "./ai-execution";

const aiErrorPayloadSchema: z.ZodType<AiError> = z
  .object({
    code: z.enum(aiErrorCodes),
    message: z.string(),
  })
  .strict();

type AiErrorPayload = AiError;

export function decodeAiErrorPayload(reason: unknown): AiErrorPayload | null {
  const parsed = aiErrorPayloadSchema.safeParse(decodeCommandError(reason));
  return parsed.success ? parsed.data : null;
}

export interface AiCompletionRequest {
  systemPrompt?: string;
  instruction: string;
  text: string;
  maxTokens?: number;
}

function aiCompletionPayload(request: AiCompletionRequest) {
  return {
    systemPrompt: request.systemPrompt ?? "",
    instruction: request.instruction,
    text: request.text,
  };
}

export async function completeGeneralWithMetadata(
  request: AiCompletionRequest,
): Promise<AiCompletionResponse> {
  const target = await loadAiExecutionTarget("knowledge");
  return invoke<AiCompletionResponse>("plugin:ai|complete", {
    request: {
      target: aiExecutionTargetPayload(target),
      maxTokens: request.maxTokens ?? null,
      ...aiCompletionPayload(request),
    } satisfies CompletionCommandRequest,
  });
}

export async function cancelAiJob(jobId: string): Promise<void> {
  return invoke<void>("plugin:ai|cancel_job", { jobId });
}
