import { Channel, invoke } from "@tauri-apps/api/core";

import { aiExecutionTargetPayload, loadAiExecutionTarget } from "./ai-execution";
import { withAiError, type AiCompletionResponse, type AiCompletionStreamEvent } from "./ai-runtime";

export type EditorTransformAction =
  | "improve"
  | "fix_grammar"
  | "extend"
  | "reduce"
  | "simplify"
  | "emojify"
  | "complete"
  | "summarize"
  | "tone_academic"
  | "tone_business"
  | "tone_casual"
  | "tone_confident"
  | "tone_creative"
  | "tone_friendly"
  | "tone_humorous"
  | "tone_informative"
  | "tone_inspirational"
  | "tone_objective"
  | "tone_persuasive";

export async function transformEditorTextStream(
  action: EditorTransformAction,
  text: string,
  jobId: string,
  onEvent: (event: AiCompletionStreamEvent) => void,
): Promise<string> {
  const target = await loadAiExecutionTarget("writing");
  const channel = new Channel<AiCompletionStreamEvent>();
  // oxlint-disable-next-line unicorn/prefer-add-event-listener -- Tauri Channel exposes events through onmessage.
  channel.onmessage = onEvent;
  const completion = await withAiError(() =>
    invoke<AiCompletionResponse>("plugin:ai|transform_editor_stream", {
      request: {
        target: aiExecutionTargetPayload(target),
        action,
        text,
      },
      jobId,
      onEvent: channel,
    }),
  );
  return completion.text;
}
