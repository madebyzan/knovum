import { computed, shallowRef } from "vue";

import { getErrorMessage } from "../../../foundation/errors/error-format";
import type { RunObservation } from "../contracts/generated/RunObservation";
import { cancelConversationTask, pauseConversationTask } from "./conversation-ipc";
import { createConversationTaskState } from "./conversation-task-state";
import {
  createConversationTaskSubmission,
  type ConversationSubmissionInput,
} from "./conversation-task-submission";
import { useConversationRunRestoration } from "./use-conversation-run-restoration";

interface ConversationTaskInput extends ConversationSubmissionInput {
  refresh: () => Promise<void>;
  select: (id: string, skipRemember?: boolean, branchId?: string) => Promise<void>;
  moveDraft: (fromId: string, toId: string) => Promise<void>;
}

export function useConversationTask(input: ConversationTaskInput) {
  const activeRun = shallowRef<RunObservation>();
  const activeRunId = shallowRef<string>();
  const activeConversationId = shallowRef("");
  const activeSpaceId = shallowRef<string>();
  const taskState = shallowRef<string>();
  const loading = shallowRef(false);
  const error = shallowRef<string>();
  const streamedText = shallowRef("");
  const activityRevision = shallowRef(0);
  const visible = computed(
    () =>
      input.spaceId.value === activeSpaceId.value &&
      input.selectedConversationId.value === activeConversationId.value,
  );
  const visibleStreamedText = computed(() => (visible.value ? streamedText.value : ""));
  const visibleTaskState = computed(() => (visible.value ? taskState.value : undefined));
  const visibleWaitReason = computed(() => (visible.value ? activeRun.value?.waitReason : undefined));
  const canPause = computed(
    () =>
      visible.value &&
      activeRun.value?.kind === "builtin_agent" &&
      (taskState.value === "running" ||
        (taskState.value === "waiting" && activeRun.value.waitReason === "executor")),
  );
  const visibleError = computed(() => (visible.value ? error.value : undefined));

  const conversation = createConversationTaskState({
    spaceId: input.spaceId,
    selectedConversationId: input.selectedConversationId,
    activeConversationId,
    refresh: input.refresh,
    select: input.select,
    moveDraft: input.moveDraft,
  });

  async function cancel() {
    if (activeRunId.value) await cancelConversationTask(activeRunId.value);
  }

  async function pause() {
    if (!canPause.value || !activeRunId.value) return;
    error.value = undefined;
    try {
      await pauseConversationTask(activeRunId.value);
    } catch (cause) {
      error.value = getErrorMessage(cause);
    }
  }

  const { restoreForSpace, resume, retryModel } = useConversationRunRestoration({
    spaceId: input.spaceId,
    selectedConversationId: input.selectedConversationId,
    activeRun,
    activeRunId,
    activeConversationId,
    activeSpaceId,
    taskState,
    loading,
    error,
    activityRevision,
    refresh: input.refresh,
    select: input.select,
    onDetach: () => submission.invalidate(),
  });

  const submission = createConversationTaskSubmission({
    input,
    state: {
      activeRun,
      activeRunId,
      activeConversationId,
      activeSpaceId,
      taskState,
      loading,
      error,
      streamedText,
      activityRevision,
    },
    accepted: conversation.accepted,
    settled: conversation.settled,
    restoreForSpace,
  });

  return {
    activeRun,
    activeRunId,
    activeConversationId,
    visibleTaskState,
    visibleWaitReason,
    canPause,
    loading,
    visibleError,
    visibleStreamedText,
    activityRevision,
    send: submission.send,
    cancel,
    pause,
    restoreForSpace,
    resume,
    retryModel,
  };
}
