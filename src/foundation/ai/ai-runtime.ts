import { invoke } from "@tauri-apps/api/core";
import { z } from "zod";

import { aiExecutionTargetPayload, loadAiExecutionTarget } from "./ai-execution";

const aiErrorPayloadSchema = z
  .object({
    code: z.enum([
      "invalid_request",
      "unsupported_provider",
      "not_configured",
      "authentication_failed",
      "capability_not_supported",
      "provider_request_failed",
      "invalid_response",
      "empty_response",
      "cancelled",
      "internal",
    ]),
    message: z.string(),
  })
  .strict();

type AiErrorPayload = z.infer<typeof aiErrorPayloadSchema>;

export function decodeAiErrorPayload(reason: unknown): AiErrorPayload | null {
  const parsed = aiErrorPayloadSchema.safeParse(reason);
  if (parsed.success) return parsed.data;
  if (!(reason instanceof Error)) return null;

  const cause = aiErrorPayloadSchema.safeParse(reason.cause);
  return cause.success ? cause.data : null;
}

export async function withAiError<T>(action: () => Promise<T>): Promise<T> {
  try {
    return await action();
  } catch (reason) {
    const decoded = decodeAiErrorPayload(reason);
    if (decoded) {
      throw new Error(decoded.message, { cause: decoded });
    }
    if (reason instanceof Error) throw reason;
    throw new Error(String(reason), { cause: reason });
  }
}

export interface AiCompletionRequest {
  systemPrompt?: string;
  instruction: string;
  text: string;
  maxTokens?: number;
}

export interface AiCompletionResponse {
  text: string;
  provider: string;
  model: string;
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
  return withAiError(() =>
    invoke<AiCompletionResponse>("plugin:ai|complete", {
      request: {
        target: aiExecutionTargetPayload(target),
        maxTokens: request.maxTokens,
        ...aiCompletionPayload(request),
      },
    }),
  );
}

export type AiCompletionStreamEvent =
  | { event: "started"; provider: string; model: string }
  | { event: "thinking_delta"; text: string }
  | { event: "delta"; text: string }
  | {
      event: "usage";
      prompt_tokens: number;
      completion_tokens: number;
      total_tokens: number;
    };

export async function cancelAiJob(jobId: string): Promise<void> {
  return invoke<void>("plugin:ai|cancel_job", { jobId });
}
