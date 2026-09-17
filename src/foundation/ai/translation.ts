import { Channel, invoke } from "@tauri-apps/api/core";

import { aiExecutionTargetPayload, loadAiExecutionTarget } from "./ai-execution";
import { decodeAiErrorPayload, type AiCompletionResponse, type AiCompletionStreamEvent } from "./ai-runtime";

export type TranslationTarget =
  | "zh-CN"
  | "zh-TW"
  | "en"
  | "ja"
  | "ko"
  | "fr"
  | "de"
  | "es"
  | "pt"
  | "it"
  | "ru"
  | "ar";

export interface TranslationRequest {
  text: string;
  target: TranslationTarget;
}

type TranslationErrorCode = "translation-unavailable" | "translation-failed";

class TranslationError extends Error {
  constructor(
    public readonly code: TranslationErrorCode,
    message: string,
  ) {
    super(message);
    this.name = "TranslationError";
  }
}

export async function translateTextStream(
  request: TranslationRequest,
  jobId: string,
  onEvent: (event: AiCompletionStreamEvent) => void,
): Promise<string> {
  if (!request.text.trim()) return "";
  try {
    const executionTarget = await loadAiExecutionTarget("translation");
    const channel = new Channel<AiCompletionStreamEvent>();
    // oxlint-disable-next-line unicorn/prefer-add-event-listener -- Tauri Channel exposes events through onmessage.
    channel.onmessage = onEvent;
    const completion = await invoke<AiCompletionResponse>("plugin:ai|translate_text_stream", {
      request: {
        executionTarget: aiExecutionTargetPayload(executionTarget),
        text: request.text,
        targetLanguage: request.target,
      },
      jobId,
      onEvent: channel,
    });
    return completion.text;
  } catch (reason) {
    throw translationError(reason);
  }
}

function translationError(reason: unknown) {
  const decoded = decodeAiErrorPayload(reason);
  const code = decoded?.code === "not_configured" ? "translation-unavailable" : "translation-failed";
  return new TranslationError(code, decoded?.message ?? "Translation failed.");
}
