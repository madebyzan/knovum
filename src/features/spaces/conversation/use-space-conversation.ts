import { computed, shallowRef, watch, type Ref } from "vue";

import type { TaskTargetRef } from "../../../shared/contracts/generated/TaskTargetRef";
import type { Conversation } from "../contracts/generated/Conversation";
import type { ConversationMessage } from "../contracts/generated/ConversationMessage";
import type { ConversationMode } from "../contracts/generated/ConversationMode";
import { useSpaceStore } from "../session/space-store";
import { createConversationSelection } from "./conversation-selection";
import { useConversationActions } from "./use-conversation-actions";
import { useConversationActivities } from "./use-conversation-activities";
import { useConversationAttachments } from "./use-conversation-attachments";
import { useConversationBranches } from "./use-conversation-branches";
import { useConversationDraftSession } from "./use-conversation-draft-session";
import { useConversationMessagePages } from "./use-conversation-message-pages";
import { useConversationTask } from "./use-conversation-task";
import { useImageCommentDraft } from "./use-image-comment-draft";

export function useSpaceConversation(spaceId: Ref<string | undefined>) {
  const store = useSpaceStore();
  const attachmentSession = useConversationAttachments();
  const { attachments, addNodeAttachments } = attachmentSession;
  const conversations = shallowRef<Conversation[]>([]);
  const selectedSnapshot = shallowRef<Conversation>();
  const messages = shallowRef<ConversationMessage[]>([]);
  const selectedConversationId = shallowRef("");
  const prompt = shallowRef("");
  const mode = shallowRef<ConversationMode>("execute");
  const target = shallowRef<TaskTargetRef>();
  const conversationQuery = shallowRef("");
  const showingArchived = shallowRef(false);
  function chooseFiles(paths?: readonly string[]) {
    const selectedSpace = spaceId.value;
    const selectedConversation = selectedConversationId.value;
    return attachmentSession.chooseFiles(
      () => spaceId.value === selectedSpace && selectedConversationId.value === selectedConversation,
      paths,
    );
  }
  const selectedConversation = computed(() =>
    selectedConversationId.value ? selectedSnapshot.value : undefined,
  );
  watch(selectedConversation, (conversation) => {
    if (conversation) mode.value = conversation.mode;
  });
  const pages = useConversationMessagePages({
    spaceId,
    selectedConversationId,
    selectedConversation,
    messages,
  });
  const draftSession = useConversationDraftSession({ spaceId, selectedConversationId, prompt, attachments });
  draftSession.restore("");
  const { refreshConversations, selectConversation } = createConversationSelection({
    spaceId,
    query: conversationQuery,
    showingArchived,
    conversations,
    selectedId: selectedConversationId,
    selectedSnapshot,
    messages,
    hasOlder: pages.hasOlder,
    rememberDraft: () => draftSession.remember(),
    restoreDraft: draftSession.restore,
    clearBranches: () => {
      branchSession.branches.value = [];
    },
    loadBranches: (id) => branchSession.load(id),
  });
  const branchSession = useConversationBranches({
    selected: selectedConversation,
    selectedId: selectedConversationId,
    get loading() {
      return task.loading;
    },
    refresh: refreshConversations,
    select: selectConversation,
  });
  useImageCommentDraft(spaceId, prompt);

  const task = useConversationTask({
    spaceId,
    selectedConversationId,
    conversation: selectedConversation,
    mode,
    prompt,
    target,
    attachments,
    visibleAttachments: attachmentSession.visibleAttachments,
    messages,
    refresh: refreshConversations,
    select: selectConversation,
    moveDraft: draftSession.move,
    clearDraft: draftSession.clearSubmitted,
    restoreDraft: draftSession.restoreSubmitted,
  });
  const {
    eventsByRun,
    error: activityError,
    inspect: inspectActivity,
  } = useConversationActivities(selectedConversationId, task.activeRunId, task.activityRevision);

  watch(
    spaceId,
    async (_, previousSpaceId) => {
      draftSession.remember(selectedConversationId.value, previousSpaceId);
      selectedConversationId.value = "";
      selectedSnapshot.value = undefined;
      messages.value = [];
      branchSession.branches.value = [];
      draftSession.restore("");
      await refreshConversations();
      if (spaceId.value) await task.restoreForSpace(spaceId.value);
    },
    { immediate: true },
  );
  watch(
    () => store.askScopeRequest,
    async (request) => {
      if (!request) return;
      store.clearAskScopeRequest();
      const requestedSpace = spaceId.value;
      if (request.restart) await selectConversation("");
      if (spaceId.value !== requestedSpace) return;
      if (request.scope === "selection") addNodeAttachments(request.nodeIds);
    },
    { immediate: true },
  );
  const conversationActions = useConversationActions({
    selected: selectedConversation,
    selectedId: selectedConversationId,
    messages,
    loading: task.loading,
    prompt,
    refresh: refreshConversations,
    select: selectConversation,
    send: task.send,
  });
  return {
    conversations,
    messages,
    branches: branchSession.branches,
    eventsByRun,
    activityError,
    inspectActivity,
    selectedConversationId,
    selectedConversation,
    mode,
    prompt,
    target,
    activeRunId: task.activeRunId,
    activeConversationId: task.activeConversationId,
    visibleTaskState: task.visibleTaskState,
    visibleWaitReason: task.visibleWaitReason,
    canPause: task.canPause,
    loading: task.loading,
    loadingOlder: pages.loadingOlder,
    hasOlder: pages.hasOlder,
    visibleError: task.visibleError,
    visibleStreamedText: task.visibleStreamedText,
    ...attachmentSession,
    chooseFiles,
    send: task.send,
    cancel: task.cancel,
    pause: task.pause,
    resume: task.resume,
    retryModel: task.retryModel,
    selectConversation,
    selectBranch: branchSession.selectBranch,
    conversationQuery,
    showingArchived,
    scrollPositions: pages.scrollPositions,
    rememberScroll: pages.rememberScroll,
    refreshConversations,
    loadOlder: pages.loadOlder,
    ...conversationActions,
  };
}
