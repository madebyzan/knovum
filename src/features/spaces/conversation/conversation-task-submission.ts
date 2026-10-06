import type { Ref, ShallowRef } from "vue";

import { getErrorMessage } from "../../../foundation/errors/error-format";
import type { TaskTargetRef } from "../../../shared/contracts/generated/TaskTargetRef";
import type { Conversation } from "../contracts/generated/Conversation";
import type { ConversationAttachment } from "../contracts/generated/ConversationAttachment";
import type { ConversationMessage } from "../contracts/generated/ConversationMessage";
import type { ConversationMode } from "../contracts/generated/ConversationMode";
import type { ConversationTaskUpdate } from "../contracts/generated/ConversationTaskUpdate";
import type { RunObservation } from "../contracts/generated/RunObservation";
import type { SubmitConversationTaskRequest } from "../contracts/generated/SubmitConversationTaskRequest";
import { getConversationTaskError, startConversationTask } from "./conversation-ipc";
import {
  pendingConversationMessage,
  type VisibleConversationAttachment,
} from "./conversation-pending-message";
import { observeConversationRun } from "./conversation-run-observation";
import { conversationTaskRequest } from "./conversation-task-request";

type StartedTask = Awaited<ReturnType<typeof startConversationTask>>;
type Selection = { spaceId: string; conversationId: string };

export interface ConversationSubmissionState {
  activeRun: ShallowRef<RunObservation | undefined>;
  activeRunId: ShallowRef<string | undefined>;
  activeConversationId: ShallowRef<string>;
  activeSpaceId: ShallowRef<string | undefined>;
  taskState: ShallowRef<string | undefined>;
  loading: ShallowRef<boolean>;
  error: ShallowRef<string | undefined>;
  streamedText: ShallowRef<string>;
  activityRevision: ShallowRef<number>;
}

export interface ConversationSubmissionInput {
  mode: Ref<ConversationMode>;
  spaceId: Ref<string | undefined>;
  selectedConversationId: Ref<string>;
  conversation: Ref<Conversation | undefined>;
  prompt: Ref<string>;
  target: Ref<TaskTargetRef | undefined>;
  attachments: Ref<ConversationAttachment[]>;
  visibleAttachments: Ref<readonly VisibleConversationAttachment[]>;
  messages: ShallowRef<ConversationMessage[]>;
  clearDraft: (request: SubmitConversationTaskRequest) => void;
  restoreDraft: (request: SubmitConversationTaskRequest) => void;
}

interface ConversationTaskSubmissionOptions {
  input: ConversationSubmissionInput;
  state: ConversationSubmissionState;
  accepted: (task: StartedTask["accepted"], selection: Selection) => Promise<void>;
  settled: (task: StartedTask["accepted"], spaceId: string) => Promise<void>;
  restoreForSpace: (spaceId: string) => Promise<void>;
}

export function createConversationTaskSubmission(options: ConversationTaskSubmissionOptions) {
  const { input, state } = options;
  let generation = 0;

  function observe(
    task: StartedTask,
    request: SubmitConversationTaskRequest,
    selection: Selection,
    submitted: number,
  ) {
    observeConversationRun({
      run: { id: task.accepted.runId, createdAt: Date.now() },
      title: input.conversation.value?.title || request.prompt.slice(0, 80),
      onUpdate: (run) => {
        if (submitted !== generation) return;
        state.activeRun.value = run;
        state.taskState.value = run.state;
        state.activityRevision.value += 1;
      },
      location: { spaceId: selection.spaceId, conversationId: task.accepted.conversationId },
    });
  }

  async function settle(task: StartedTask, selection: Selection, submitted: number) {
    try {
      if (submitted === generation) await options.accepted(task.accepted, selection);
    } catch (reason) {
      if (submitted === generation) state.error.value = getErrorMessage(reason);
    }
    const outcome = await task.settled;
    if (submitted !== generation) return;
    state.taskState.value = outcome;
    await options.settled(task.accepted, selection.spaceId);
    state.streamedText.value = "";
    state.error.value = undefined;
    if (outcome === "succeeded" || outcome === "paused" || outcome === "waiting" || outcome === "cancelled")
      return;
    const reason = await getConversationTaskError(task.accepted.runId);
    throw new Error(reason ?? `Conversation task ended as ${outcome}`);
  }

  async function finish(submitted: number) {
    if (submitted !== generation) return;
    state.loading.value = false;
    if (
      (state.taskState.value === "paused" || state.taskState.value === "waiting") &&
      state.activeSpaceId.value
    ) {
      state.activeRunId.value = undefined;
      try {
        await options.restoreForSpace(state.activeSpaceId.value);
      } catch (cause) {
        state.error.value = getErrorMessage(cause);
      }
    } else if (state.taskState.value !== "waiting") {
      state.activeRunId.value = undefined;
    }
  }

  function handleUpdate(update: ConversationTaskUpdate, submitted: number) {
    if (submitted !== generation) return;
    if (update.type === "content_delta") state.streamedText.value += update.text;
    else if (update.type === "content_boundary") state.streamedText.value = "";
  }

  function clearPending(id: string) {
    if (input.messages.value.some((message) => message.id === id))
      input.messages.value = input.messages.value.filter((message) => message.id !== id);
  }

  async function send() {
    if (state.loading.value || state.activeRunId.value) return;
    const request = conversationTaskRequest({
      mode: input.mode.value,
      spaceId: input.spaceId.value,
      conversation: input.conversation.value,
      prompt: input.prompt.value,
      target: input.target.value,
      attachments: input.attachments.value,
    });
    if (!request) return;
    const submitted = ++generation;
    let accepted = false;
    const selection = { spaceId: request.spaceId, conversationId: input.selectedConversationId.value };
    const pendingId = `pending:${crypto.randomUUID()}`;
    const pendingMessage = pendingConversationMessage(
      request,
      {
        id: pendingId,
        conversationId: selection.conversationId,
        branchId: input.conversation.value?.branchId ?? "",
      },
      input.visibleAttachments.value,
    );
    input.clearDraft(request);
    input.messages.value = [...input.messages.value, pendingMessage];
    state.activeSpaceId.value = selection.spaceId;
    state.activeConversationId.value = selection.conversationId;
    state.loading.value = true;
    state.taskState.value = undefined;
    state.error.value = undefined;
    state.streamedText.value = "";
    try {
      const task = await startConversationTask(request, (update) => handleUpdate(update, submitted));
      accepted = true;
      if (submitted !== generation) return;
      state.activeRunId.value = task.accepted.runId;
      observe(task, request, selection, submitted);
      await settle(task, selection, submitted);
    } catch (reason) {
      if (submitted === generation) {
        if (!accepted) input.restoreDraft(request);
        state.error.value = getErrorMessage(reason);
      }
    } finally {
      clearPending(pendingId);
      await finish(submitted);
    }
  }

  return {
    send,
    invalidate: () => {
      generation += 1;
    },
  };
}
