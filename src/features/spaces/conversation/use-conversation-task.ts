import { computed, shallowRef, type Ref } from "vue";

import type { AssistantTaskUpdate } from "../contracts/generated/AssistantTaskUpdate";
import type { WaitReason } from "../contracts/generated/WaitReason";
import {
  cancelAssistantTask,
  type ConversationSummary,
  type StartAssistantTaskInput,
} from "./conversation-ipc";
import type { ConversationTarget } from "./conversation-target";
import { createConversationTaskState } from "./conversation-task-state";
import { createConversationTaskSubmission } from "./conversation-task-submission";
import { useConversationRunRestoration } from "./use-conversation-run-restoration";

type Attachment = NonNullable<StartAssistantTaskInput["attachments"]>[number];
type PermissionRequest = Extract<AssistantTaskUpdate, { type: "permission_requested" }>;

export function useConversationTask(input: {
  spaceId: Ref<string | undefined>;
  selectedConversationId: Ref<string>;
  conversation: Ref<ConversationSummary | undefined>;
  prompt: Ref<string>;
  target: Ref<ConversationTarget | undefined>;
  attachments: Ref<Attachment[]>;
  refresh: () => Promise<void>;
  select: (id: string, skipRemember?: boolean, branchId?: string) => Promise<void>;
  moveDraft: (fromId: string, toId: string) => void;
  clearDraft: (request: StartAssistantTaskInput) => void;
}) {
  const activeRunId = shallowRef<string>();
  const activeConversationId = shallowRef("");
  const activeSpaceId = shallowRef<string>();
  const taskState = shallowRef<string>();
  const waitReason = shallowRef<WaitReason | null>(null);
  const loading = shallowRef(false);
  const error = shallowRef<string>();
  const streamedText = shallowRef("");
  const permission = shallowRef<PermissionRequest>();
  const visible = computed(
    () =>
      input.spaceId.value === activeSpaceId.value &&
      input.selectedConversationId.value === activeConversationId.value,
  );
  const visibleStreamedText = computed(() => (visible.value ? streamedText.value : ""));
  const visibleTaskState = computed(() => (visible.value ? taskState.value : undefined));
  const visibleWaitReason = computed(() => (visible.value ? waitReason.value : null));
  const visibleError = computed(() => (visible.value ? error.value : undefined));
  const visiblePermission = computed(() => (visible.value ? permission.value : undefined));

  const conversation = createConversationTaskState({
    spaceId: input.spaceId,
    selectedConversationId: input.selectedConversationId,
    activeConversationId,
    refresh: input.refresh,
    select: input.select,
    moveDraft: input.moveDraft,
  });

  async function cancel() {
    if (activeRunId.value) await cancelAssistantTask(activeRunId.value);
  }

  const {
    restoreForSpace,
    resume,
    decidingPermission,
    decidePermission,
    approvals,
    decidingApproval,
    decideApproval,
    refreshPending,
  } = useConversationRunRestoration({
    spaceId: input.spaceId,
    selectedConversationId: input.selectedConversationId,
    activeRunId,
    activeConversationId,
    activeSpaceId,
    taskState,
    waitReason,
    loading,
    permission,
    error,
    refresh: input.refresh,
    select: input.select,
    onDetach: () => submission.invalidate(),
  });

  const submission = createConversationTaskSubmission({
    input,
    state: {
      activeRunId,
      activeConversationId,
      activeSpaceId,
      taskState,
      waitReason,
      loading,
      error,
      streamedText,
      permission,
    },
    accepted: conversation.accepted,
    settled: conversation.settled,
    refreshPending,
    restoreForSpace,
  });

  return {
    activeRunId,
    activeConversationId,
    pendingPermission: permission,
    visibleTaskState,
    visibleWaitReason,
    loading,
    visibleError,
    visiblePermission,
    decidingPermission,
    approvals,
    decidingApproval,
    decideApproval,
    visibleStreamedText,
    send: submission.send,
    cancel,
    decidePermission,
    restoreForSpace,
    resume,
  };
}
