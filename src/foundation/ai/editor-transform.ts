import { Channel } from "@tauri-apps/api/core";

import type { EditorTransformAction } from "../../shared/contracts/generated/EditorTransformAction";
import type { EditorTransformCommandRequest } from "../../shared/contracts/generated/EditorTransformCommandRequest";
import type { TaskCompletionOutput } from "../../shared/contracts/generated/TaskCompletionOutput";
import type { TaskStreamEvent } from "../../shared/contracts/generated/TaskStreamEvent";
import { invokeCommand } from "../ipc/invoke-command";
import { aiExecutionTargetPayload, loadAiExecutionTarget } from "./ai-execution";

interface EditorTextTransformInput {
  action: EditorTransformAction;
  text: string;
  jobId: string;
  onEvent: (event: TaskStreamEvent) => void;
  signal: AbortSignal;
}

export async function transformEditorTextStream({
  action,
  text,
  jobId,
  onEvent,
  signal,
}: EditorTextTransformInput): Promise<string> {
  signal.throwIfAborted();
  const target = await loadAiExecutionTarget("writing");
  signal.throwIfAborted();
  const channel = new Channel<TaskStreamEvent>();
  // oxlint-disable-next-line unicorn/prefer-add-event-listener -- Tauri Channel exposes events through onmessage.
  channel.onmessage = onEvent;
  const completion = await invokeCommand<TaskCompletionOutput>("plugin:ai|transform_editor_text_stream", {
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
