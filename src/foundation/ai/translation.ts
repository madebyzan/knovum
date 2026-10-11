import { Channel } from "@tauri-apps/api/core";

import type { TaskCompletionOutput } from "../../shared/contracts/generated/TaskCompletionOutput";
import type { TaskStreamEvent } from "../../shared/contracts/generated/TaskStreamEvent";
import type { TranslationCommandRequest } from "../../shared/contracts/generated/TranslationCommandRequest";
import { invokeCommand } from "../ipc/invoke-command";
import { aiExecutionTargetPayload, loadAiExecutionTarget } from "./ai-execution";

export const TRANSLATION_TARGETS = [
  "zh-CN",
  "zh-TW",
  "en",
  "ja",
  "ko",
  "fr",
  "de",
  "es",
  "pt",
  "it",
  "ru",
  "ar",
] as const;

export type TranslationTarget = (typeof TRANSLATION_TARGETS)[number];

export const TRANSLATION_TARGET_LABEL_KEYS: Record<TranslationTarget, string> = {
  "zh-CN": "ai.languages.zhCn",
  "zh-TW": "ai.languages.zhTw",
  en: "ai.languages.en",
  ja: "ai.languages.ja",
  ko: "ai.languages.ko",
  fr: "ai.languages.fr",
  de: "ai.languages.de",
  es: "ai.languages.es",
  pt: "ai.languages.pt",
  it: "ai.languages.it",
  ru: "ai.languages.ru",
  ar: "ai.languages.ar",
};

export interface TranslationInput {
  text: string;
  target: TranslationTarget;
}

export async function translateTextStream(
  request: TranslationInput,
  jobId: string,
  onEvent: (event: TaskStreamEvent) => void,
  signal: AbortSignal,
): Promise<string> {
  signal.throwIfAborted();
  if (!request.text.trim()) return "";
  const executionTarget = await loadAiExecutionTarget("translation");
  signal.throwIfAborted();
  const channel = new Channel<TaskStreamEvent>();
  // oxlint-disable-next-line unicorn/prefer-add-event-listener -- Tauri Channel exposes events through onmessage.
  channel.onmessage = onEvent;
  const completion = await invokeCommand<TaskCompletionOutput>("plugin:ai|translate_text_stream", {
    request: {
      executionTarget: aiExecutionTargetPayload(executionTarget),
      text: request.text,
      targetLanguage: request.target,
    } satisfies TranslationCommandRequest,
    jobId,
    onEvent: channel,
  });
  return completion.text;
}
