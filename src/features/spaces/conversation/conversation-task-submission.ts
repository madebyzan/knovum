import type { Ref, ShallowRef } from "vue";

import { getErrorMessage } from "../../../foundation/errors/error-format";
import type { TaskTargetRef } from "../../../shared/contracts/generated/TaskTargetRef";
import type { AssistantConversation } from "../contracts/generated/AssistantConversation";
import type { AssistantMessage } from "../contracts/generated/AssistantMessage";
import type { AssistantTaskUpdate } from "../contracts/generated/AssistantTaskUpdate";
import type { SubmitAssistantTaskRequest } from "../contracts/generated/SubmitAssistantTaskRequest";
import { getAssistantTaskError, startAssistantTask } from "./conversation-ipc";
import { pendingConversationMessage } from "./conversation-pending-message";
import { observeConversationRun } from "./conversation-run-observation";
import { conversationTaskRequest } from "./conversation-task-request";

type Attachment = NonNullable<SubmitAssistantTaskRequest["attachments"]>[number];
type StartedTask = Awaited<ReturnType<typeof startAssistantTask>>;
type Selection = { spaceId: string; conversationId: string };

export interface ConversationSubmissionState {
  activeRunId: ShallowRef<string | undefined>;
  activeConversationId: ShallowRef<string>;
  activeSpaceId: ShallowRef<string | undefined>;
  taskState: ShallowRef<string | undefined>;
  loading: ShallowRef<boolean>;
  error: ShallowRef<string | undefined>;
  streamedText: ShallowRef<string>;
  activityRevision: ShallowRef<number>;
}

export function createConversationTaskSubmission(options: {
  input: {
    spaceId: Ref<string | undefined>;
    selectedConversationId: Ref<string>;
    conversation: Ref<AssistantConversation | undefined>;
    prompt: Ref<string>;
    target: Ref<TaskTargetRef | undefined>;
    attachments: Ref<Attachment[]>;
    visibleAttachments: Ref<readonly { title: string; isImage: boolean; previewSource: string }[]>;
    messages: ShallowRef<AssistantMessage[]>;
    clearDraft: (request: SubmitAssistantTaskRequest) => void;
    restoreDraft: (request: SubmitAssistantTaskRequest) => void;
  };
  state: ConversationSubmissionState;
  accepted: (task: StartedTask["accepted"], selection: Selection) => Promise<void>;
  settled: (task: StartedTask["accepted"], spaceId: string) => Promise<void>;
  restoreForSpace: (spaceId: string) => Promise<void>;
}) {
  const { input, state } = options;
  let generation = 0;

  function observe(
    task: StartedTask,
    request: SubmitAssistantTaskRequest,
    selection: Selection,
    submitted: number,
  ) {
    observeConversationRun(
      { id: task.accepted.runId, createdAt: Date.now() },
      input.conversation.value?.title || request.prompt.slice(0, 80),
      (run) => {
        if (submitted !== generation) return;
        state.taskState.value = run.state;
        state.activityRevision.value += 1;
      },
      { spaceId: selection.spaceId, conversationId: task.accepted.conversationId },
    );
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
    if (outcome === "succeeded" || outcome === "paused" || outcome === "cancelled") return;
    const reason = await getAssistantTaskError(task.accepted.runId);
    throw new Error(reason ?? `Assistant task ended as ${outcome}`);
  }

  async function finish(submitted: number) {
    if (submitted !== generation) return;
    state.loading.value = false;
    if (state.taskState.value === "paused" && state.activeSpaceId.value) {
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

  function handleUpdate(update: AssistantTaskUpdate, submitted: number) {
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
      const task = await startAssistantTask(request, (update) => handleUpdate(update, submitted));
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
