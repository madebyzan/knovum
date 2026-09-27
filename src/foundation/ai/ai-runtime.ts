import { z } from "zod";

import type { AiError } from "../../shared/contracts/generated/AiError";
import { aiErrorCodes } from "../../shared/contracts/generated/contract-values";
import { decodeCommandError } from "../ipc/command-error";
import { invokeCommand } from "../ipc/invoke-command";

const aiErrorPayloadSchema: z.ZodType<AiError> = z
  .object({
    code: z.enum(aiErrorCodes),
    message: z.string(),
  })
  .strict();

export function decodeAiErrorPayload(reason: unknown): AiError | null {
  const parsed = aiErrorPayloadSchema.safeParse(decodeCommandError(reason));
  return parsed.success ? parsed.data : null;
}

export async function cancelAiJob(jobId: string): Promise<void> {
  return invokeCommand<void>("plugin:ai|cancel_job", { jobId });
}
