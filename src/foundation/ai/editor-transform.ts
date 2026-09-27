import type { CompletionOutput } from "../../shared/contracts/generated/CompletionOutput";
import type { CompletionStreamEvent } from "../../shared/contracts/generated/CompletionStreamEvent";
import type { EditorTransformAction } from "../../shared/contracts/generated/EditorTransformAction";
import type { EditorTransformCommandRequest } from "../../shared/contracts/generated/EditorTransformCommandRequest";
export type { EditorTransformAction } from "../../shared/contracts/generated/EditorTransformAction";
import { Channel } from "@tauri-apps/api/core";

import { invokeCommand } from "../ipc/invoke-command";
import { aiExecutionTargetPayload, loadAiExecutionTarget } from "./ai-execution";

export async function transformEditorTextStream(
  action: EditorTransformAction,
  text: string,
  jobId: string,
  onEvent: (event: CompletionStreamEvent) => void,
): Promise<string> {
  const target = await loadAiExecutionTarget("writing");
  const channel = new Channel<CompletionStreamEvent>();
  // oxlint-disable-next-line unicorn/prefer-add-event-listener -- Tauri Channel exposes events through onmessage.
  channel.onmessage = onEvent;
  const completion = await invokeCommand<CompletionOutput>("plugin:ai|transform_editor_text_stream", {
    request: {
      target: aiExecutionTargetPayload(target),
      action,
      text,
    } satisfies EditorTransformCommandRequest,
    jobId,
    onEvent: channel,
  });
  return completion.text;
}
