import { computed, shallowRef, type Ref, type ShallowRef } from "vue";

import {
  cancelAssistantTask,
  type AssistantMessage,
  type AssistantConversation,
  type StartAssistantTaskInput,
} from "./conversation-ipc";
import type { ConversationTarget } from "./conversation-target";
import { createConversationTaskState } from "./conversation-task-state";
import { createConversationTaskSubmission } from "./conversation-task-submission";
import { useConversationRunRestoration } from "./use-conversation-run-restoration";

type Attachment = NonNullable<StartAssistantTaskInput["attachments"]>[number];

export function useConversationTask(input: {
  spaceId: Ref<string | undefined>;
  selectedConversationId: Ref<string>;
  conversation: Ref<AssistantConversation | undefined>;
  prompt: Ref<string>;
  target: Ref<ConversationTarget | undefined>;
  attachments: Ref<Attachment[]>;
  visibleAttachments: Ref<readonly { title: string; isImage: boolean; previewSource: string }[]>;
  messages: ShallowRef<AssistantMessage[]>;
  refresh: () => Promise<void>;
  select: (id: string, skipRemember?: boolean, branchId?: string) => Promise<void>;
  moveDraft: (fromId: string, toId: string) => void;
  clearDraft: (request: StartAssistantTaskInput) => void;
  restoreDraft: (request: StartAssistantTaskInput) => void;
}) {
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
    if (activeRunId.value) await cancelAssistantTask(activeRunId.value);
  }

  const { restoreForSpace, resume } = useConversationRunRestoration({
    spaceId: input.spaceId,
    selectedConversationId: input.selectedConversationId,
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
    activeRunId,
    activeConversationId,
    visibleTaskState,
    loading,
    visibleError,
    visibleStreamedText,
    activityRevision,
    send: submission.send,
    cancel,
    restoreForSpace,
    resume,
  };
}
