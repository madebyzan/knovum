import { Channel } from "@tauri-apps/api/core";

import type { NarrationPreparationCommandRequest } from "../../shared/contracts/generated/NarrationPreparationCommandRequest";
import type { NarrationPreparationMode } from "../../shared/contracts/generated/NarrationPreparationMode";
import type { TaskCompletionOutput } from "../../shared/contracts/generated/TaskCompletionOutput";
import type { TaskStreamEvent } from "../../shared/contracts/generated/TaskStreamEvent";
import type { TaskTarget } from "../../shared/contracts/generated/TaskTarget";
import { invokeCommand } from "../ipc/invoke-command";
import { aiExecutionTargetPayload, loadAiExecutionTarget } from "./ai-execution";

export interface NarrationPreparationInput {
  mode: NarrationPreparationMode;
  text: string;
  jobId: string;
  target?: TaskTarget | undefined;
  onEvent: (event: TaskStreamEvent) => void;
  signal: AbortSignal;
}

export async function prepareNarrationScriptStream({
  mode,
  text,
  jobId,
  target,
  onEvent,
  signal,
}: NarrationPreparationInput): Promise<string> {
  signal.throwIfAborted();
  const resolved = await loadAiExecutionTarget("writing", target);
  signal.throwIfAborted();
  const channel = new Channel<TaskStreamEvent>();
  // oxlint-disable-next-line unicorn/prefer-add-event-listener -- Tauri Channel exposes events through onmessage.
  channel.onmessage = onEvent;
  const request: NarrationPreparationCommandRequest = {
    target: aiExecutionTargetPayload(resolved),
    mode,
    text,
  };
  const completion = await invokeCommand<TaskCompletionOutput>("plugin:ai|prepare_narration_script_stream", {
    request,
    jobId,
    onEvent: channel,
  });
  return completion.text;
}
