import { Channel } from "@tauri-apps/api/core";

import type { CompletionOutput } from "../../shared/contracts/generated/CompletionOutput";
import type { CompletionStreamEvent } from "../../shared/contracts/generated/CompletionStreamEvent";
import type { TranslationCommandRequest } from "../../shared/contracts/generated/TranslationCommandRequest";
import { invokeCommand } from "../ipc/invoke-command";
import { aiExecutionTargetPayload, loadAiExecutionTarget } from "./ai-execution";
import { decodeAiErrorPayload } from "./ai-runtime";

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

export interface TranslationInput {
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
  request: TranslationInput,
  jobId: string,
  onEvent: (event: CompletionStreamEvent) => void,
): Promise<string> {
  if (!request.text.trim()) return "";
  try {
    const executionTarget = await loadAiExecutionTarget("translation");
    const channel = new Channel<CompletionStreamEvent>();
    // oxlint-disable-next-line unicorn/prefer-add-event-listener -- Tauri Channel exposes events through onmessage.
    channel.onmessage = onEvent;
    const completion = await invokeCommand<CompletionOutput>("plugin:ai|translate_text_stream", {
      request: {
        executionTarget: aiExecutionTargetPayload(executionTarget),
        text: request.text,
        targetLanguage: request.target,
      } satisfies TranslationCommandRequest,
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
