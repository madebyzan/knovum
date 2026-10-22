import { getErrorMessage } from "../../../foundation/errors/error-format";
import type { ConversationTaskUpdate } from "../contracts/generated/ConversationTaskUpdate";
import type { SubmitConversationTaskRequest } from "../contracts/generated/SubmitConversationTaskRequest";
import { restoreConversationDraftAfterFailure } from "./conversation-draft-recovery";
import { startConversationTask } from "./conversation-ipc";
import { pendingConversationMessage } from "./conversation-pending-message";
import { observeConversationRun } from "./conversation-run-observation";
import { conversationTaskRequest } from "./conversation-task-request";
import { settleConversationTask } from "./conversation-task-settlement";
import type {
  ConversationSubmissionInput,
  ConversationSubmissionState,
} from "./conversation-task-submission-contract";

type StartedTask = Awaited<ReturnType<typeof startConversationTask>>;
type Selection = { spaceId: string; conversationId: string };
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
    state.error.value = undefined;
    await settleConversationTask({
      accepted: task.accepted,
      outcome,
      spaceId: selection.spaceId,
      project: async (accepted, spaceId) => {
        // The streamed answer stays on screen until the stored one has been loaded to replace it;
        // clearing it first would flash whatever partial text the previous load held.
        try {
          await options.settled(accepted, spaceId);
        } finally {
          if (submitted === generation) state.streamedText.value = "";
        }
      },
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
      await input.ensureSpacePersisted();
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
