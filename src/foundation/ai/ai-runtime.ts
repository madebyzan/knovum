import type { AiError } from "../../shared/contracts/generated/AiError";
import { aiErrorCodes } from "../../shared/contracts/generated/contract-values";
export type { CompletionOutput as AiCompletionResponse } from "../../shared/contracts/generated/CompletionOutput";

export type { CompletionStreamEvent as AiCompletionStreamEvent } from "../../shared/contracts/generated/CompletionStreamEvent";
import { z } from "zod";

import { decodeCommandError } from "../ipc/command-error";
import { invokeCommand as invoke } from "../ipc/invoke-command";

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

export async function cancelAiJob(jobId: string): Promise<void> {
  return invoke<void>("plugin:ai|cancel_job", { jobId });
}
