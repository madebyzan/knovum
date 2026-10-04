import { Channel } from "@tauri-apps/api/core";

import type { TaskCompletionOutput } from "../../shared/contracts/generated/TaskCompletionOutput";
import type { TaskStreamEvent } from "../../shared/contracts/generated/TaskStreamEvent";
import type { TranslationCommandRequest } from "../../shared/contracts/generated/TranslationCommandRequest";
import { invokeCommand } from "../ipc/invoke-command";
import { aiExecutionTargetPayload, loadAiExecutionTarget } from "./ai-execution";

export const translationTargets = [
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

export type TranslationTarget = (typeof translationTargets)[number];

export const translationTargetLabelKey: Record<TranslationTarget, string> = {
  "zh-CN": "spaces.richtext.ai.languages.zhCn",
  "zh-TW": "spaces.richtext.ai.languages.zhTw",
  en: "spaces.richtext.ai.languages.en",
  ja: "spaces.richtext.ai.languages.ja",
  ko: "spaces.richtext.ai.languages.ko",
  fr: "spaces.richtext.ai.languages.fr",
  de: "spaces.richtext.ai.languages.de",
  es: "spaces.richtext.ai.languages.es",
  pt: "spaces.richtext.ai.languages.pt",
  it: "spaces.richtext.ai.languages.it",
  ru: "spaces.richtext.ai.languages.ru",
  ar: "spaces.richtext.ai.languages.ar",
};

export interface TranslationInput {
  text: string;
  target: TranslationTarget;
}

export async function translateTextStream(
  request: TranslationInput,
  jobId: string,
  onEvent: (event: TaskStreamEvent) => void,
): Promise<string> {
  if (!request.text.trim()) return "";
  const executionTarget = await loadAiExecutionTarget("translation");
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
