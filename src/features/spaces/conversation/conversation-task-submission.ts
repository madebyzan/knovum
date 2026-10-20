import type { Ref, ShallowRef } from "vue";

import { getErrorMessage } from "../../../foundation/errors/error-format";
import type { AgentExtensionSelection } from "../../../shared/contracts/generated/AgentExtensionSelection";
import type { TaskTarget } from "../../../shared/contracts/generated/TaskTarget";
import type { Conversation } from "../contracts/generated/Conversation";
import type { ConversationAttachment } from "../contracts/generated/ConversationAttachment";
import type { ConversationMessage } from "../contracts/generated/ConversationMessage";
import type { ConversationTaskUpdate } from "../contracts/generated/ConversationTaskUpdate";
import type { RunObservation } from "../contracts/generated/RunObservation";
import type { SubmitConversationTaskRequest } from "../contracts/generated/SubmitConversationTaskRequest";
import { restoreConversationDraftAfterFailure } from "./conversation-draft-recovery";
import { startConversationTask } from "./conversation-ipc";
import {
  pendingConversationMessage,
  type VisibleConversationAttachment,
} from "./conversation-pending-message";
import { observeConversationRun } from "./conversation-run-observation";
import { conversationTaskRequest } from "./conversation-task-request";
import { settleConversationTask } from "./conversation-task-settlement";
type StartedTask = Awaited<ReturnType<typeof startConversationTask>>;
type Selection = { spaceId: string; conversationId: string };
interface ConversationSubmissionState {
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
  extensions: Ref<AgentExtensionSelection>;
  spaceId: Ref<string | undefined>;
  selectedConversationId: Ref<string>;
  conversation: Ref<Conversation | undefined>;
  prompt: Ref<string>;
  target: Ref<TaskTarget | undefined>;
  attachments: Ref<ConversationAttachment[]>;
  visibleAttachments: Ref<readonly VisibleConversationAttachment[]>;
  hasUnresolvedFileReferences: Ref<boolean>;
  messages: ShallowRef<ConversationMessage[]>;
  clearDraft: (request: SubmitConversationTaskRequest) => Promise<void>;
  restoreDraft: (request: SubmitConversationTaskRequest) => Promise<void>;
}
interface ConversationTaskSubmissionOptions {
  input: ConversationSubmissionInput;
  state: ConversationSubmissionState;
  accepted: (task: StartedTask["accepted"], selection: Selection) => Promise<void>;
  settled: (task: StartedTask["accepted"], spaceId: string) => Promise<void>;
  restoreForSpace: (spaceId: string) => Promise<void>;
}

function canSubmitConversation(input: ConversationSubmissionInput, state: ConversationSubmissionState) {
  return !state.loading.value && !state.activeRunId.value && !input.hasUnresolvedFileReferences.value;
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
    state.taskState.value = outcome.state;
    state.streamedText.value = "";
    state.error.value = undefined;
    await settleConversationTask({
      accepted: task.accepted,
      outcome,
      spaceId: selection.spaceId,
      project: options.settled,
    });
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

  function clearPending(messageId: string) {
    if (input.messages.value.some((message) => message.id === messageId))
      input.messages.value = input.messages.value.filter((message) => message.id !== messageId);
  }

  async function send() {
    if (!canSubmitConversation(input, state)) return;
    const request = conversationTaskRequest({
      spaceId: input.spaceId.value,
      conversation: input.conversation.value,
      prompt: input.prompt.value,
      target: input.target.value,
      attachments: input.attachments.value,
      extensions: input.extensions.value,
    });
    if (!request) return;
    const submitted = ++generation;
    let accepted = false;
    const selection = { spaceId: request.spaceId, conversationId: input.selectedConversationId.value };
    const pendingMessageId = `pending:${crypto.randomUUID()}`;
    const pendingMessage = pendingConversationMessage(
      request,
      {
        id: pendingMessageId,
        conversationId: selection.conversationId,
        branchId: input.conversation.value?.branchId ?? "",
      },
      input.visibleAttachments.value,
    );
    state.activeSpaceId.value = selection.spaceId;
    state.activeConversationId.value = selection.conversationId;
    state.loading.value = true;
    state.taskState.value = undefined;
    state.error.value = undefined;
    state.streamedText.value = "";
    try {
      await input.clearDraft(request);
      input.messages.value = [...input.messages.value, pendingMessage];
      const task = await startConversationTask(request, (update) => handleUpdate(update, submitted));
      accepted = true;
      if (submitted !== generation) return;
      state.activeRunId.value = task.accepted.runId;
      observe(task, request, selection, submitted);
      await settle(task, selection, submitted);
    } catch (reason) {
      if (submitted === generation) {
        const failure = accepted
          ? reason
          : await restoreConversationDraftAfterFailure(input.restoreDraft, request, reason);
        state.error.value = getErrorMessage(failure);
      }
    } finally {
      clearPending(pendingMessageId);
      await finish(submitted);
    }
  }

  return { send, invalidate: () => (generation += 1) };
}
