import { Channel } from "@tauri-apps/api/core";

import type { EditorDocumentContext } from "../../shared/contracts/generated/EditorDocumentContext";
import type { EditorTransformAction } from "../../shared/contracts/generated/EditorTransformAction";
import type { EditorTransformCommandRequest } from "../../shared/contracts/generated/EditorTransformCommandRequest";
import type { TaskCompletionOutput } from "../../shared/contracts/generated/TaskCompletionOutput";
import type { TaskStreamEvent } from "../../shared/contracts/generated/TaskStreamEvent";
import type { TaskTarget } from "../../shared/contracts/generated/TaskTarget";
import { invokeCommand } from "../ipc/invoke-command";
import { aiExecutionTargetPayload, loadAiExecutionTarget } from "./ai-execution";

export interface EditorTextTransformInput {
  action: EditorTransformAction;
  text: string;
  jobId: string;
  target?: TaskTarget | undefined;
  instruction?: string | undefined;
  context?: EditorDocumentContext | undefined;
  previousDraft?: string | undefined;
  onEvent: (event: TaskStreamEvent) => void;
  signal: AbortSignal;
}

export async function transformEditorTextStream({
  action,
  text,
  jobId,
  target,
  instruction,
  context,
  previousDraft,
  onEvent,
  signal,
}: EditorTextTransformInput): Promise<string> {
  signal.throwIfAborted();
  const resolved = await loadAiExecutionTarget("writing", target);
  signal.throwIfAborted();
  const channel = new Channel<TaskStreamEvent>();
  // oxlint-disable-next-line unicorn/prefer-add-event-listener -- Tauri Channel exposes events through onmessage.
  channel.onmessage = onEvent;
  const request: EditorTransformCommandRequest = {
    target: aiExecutionTargetPayload(resolved),
    action,
    text,
    ...(instruction ? { instruction } : {}),
    ...(context ? { context } : {}),
    ...(previousDraft ? { previousDraft } : {}),
  };
  const completion = await invokeCommand<TaskCompletionOutput>("plugin:ai|transform_editor_text_stream", {
    request,
    jobId,
    onEvent: channel,
  });
  return completion.text;
}
