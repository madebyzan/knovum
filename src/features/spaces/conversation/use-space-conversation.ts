import { computed, shallowRef, watch, type Ref } from "vue";

import { useSpaceStore } from "../session/space-store";
import {
  listAssistantConversations,
  listAssistantMessages,
  type AssistantConversation,
  type AssistantMessage,
} from "./conversation-ipc";
import type { ConversationTarget } from "./conversation-target";
import { useConversationActions } from "./use-conversation-actions";
import { useConversationAttachments } from "./use-conversation-attachments";
import { useConversationDraftSession } from "./use-conversation-draft-session";
import { useConversationTask } from "./use-conversation-task";
import { useImageCommentDraft } from "./use-image-comment-draft";

export function useSpaceConversation(spaceId: Ref<string | undefined>) {
  const store = useSpaceStore();
  const attachmentSession = useConversationAttachments();
  const { attachments, addNodeAttachments } = attachmentSession;
  const conversations = shallowRef<AssistantConversation[]>([]);
  const messages = shallowRef<AssistantMessage[]>([]);
  const selectedConversationId = shallowRef("");
  const prompt = shallowRef("");
  const target = shallowRef<ConversationTarget>();
  const loadingOlder = shallowRef(false);
  const hasOlder = shallowRef(false);
  const conversationQuery = shallowRef("");
  const showingArchived = shallowRef(false);
  const scrollPositions = shallowRef<Record<string, number>>({});
  function chooseFiles() {
    const selectedSpace = spaceId.value;
    const selectedConversation = selectedConversationId.value;
    return attachmentSession.chooseFiles(
      () => spaceId.value === selectedSpace && selectedConversationId.value === selectedConversation,
    );
  }
  const selectedConversation = computed(() =>
    conversations.value.find(({ id }) => id === selectedConversationId.value),
  );

  const draftSession = useConversationDraftSession({ spaceId, selectedConversationId, prompt, attachments });
  draftSession.restore("");
  useImageCommentDraft(spaceId, prompt);
  async function refreshConversations() {
    const requestedSpaceId = spaceId.value;
    if (!requestedSpaceId) return;
    const loadedConversations = await listAssistantConversations(
      requestedSpaceId,
      conversationQuery.value,
      showingArchived.value,
    );
    if (spaceId.value === requestedSpaceId) conversations.value = loadedConversations;
  }

  async function selectConversation(id: string, skipRemember = false, acceptedBranchId?: string) {
    if (!skipRemember) draftSession.remember();
    selectedConversationId.value = id;
    draftSession.restore(id);
    if (!id) {
      messages.value = [];
      hasOlder.value = false;
      return;
    }
    const conversation = conversations.value.find((item) => item.id === id);
    const branchId = conversation?.branchId ?? acceptedBranchId;
    const requestedSpaceId = spaceId.value;
    if (!branchId) return;
    const page = await listAssistantMessages(id, branchId);
    if (selectedConversationId.value !== id || spaceId.value !== requestedSpaceId) return;
    messages.value = page;
    hasOlder.value = page.length === 100;
  }

  async function loadOlder() {
    const conversation = selectedConversation.value;
    const first = messages.value[0];
    if (!conversation?.branchId || !first || loadingOlder.value || !hasOlder.value) return;
    loadingOlder.value = true;
    try {
      const requestedSpaceId = spaceId.value;
      const page = await listAssistantMessages(conversation.id, conversation.branchId, first);
      if (spaceId.value !== requestedSpaceId || selectedConversationId.value !== conversation.id) return;
      messages.value = [...page, ...messages.value];
      hasOlder.value = page.length === 100;
    } finally {
      loadingOlder.value = false;
    }
  }

  function rememberScroll(id: string, top: number) {
    scrollPositions.value = { ...scrollPositions.value, [id]: top };
  }

  const task = useConversationTask({
    spaceId,
    selectedConversationId,
    conversation: selectedConversation,
    prompt,
    target,
    attachments,
    messages,
    refresh: refreshConversations,
    select: selectConversation,
    moveDraft: draftSession.move,
    clearDraft: draftSession.clearSubmitted,
  });

  watch(
    spaceId,
    async (_, previousSpaceId) => {
      draftSession.remember(selectedConversationId.value, previousSpaceId);
      selectedConversationId.value = "";
      messages.value = [];
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
    selectedConversationId,
    prompt,
    target,
    activeRunId: task.activeRunId,
    activeConversationId: task.activeConversationId,
    pendingPermission: task.pendingPermission,
    approvals: task.approvals,
    decidingApproval: task.decidingApproval,
    visibleTaskState: task.visibleTaskState,
    visibleWaitReason: task.visibleWaitReason,
    loading: task.loading,
    loadingOlder,
    hasOlder,
    visibleError: task.visibleError,
    visiblePermission: task.visiblePermission,
    decidingPermission: task.decidingPermission,
    visibleStreamedText: task.visibleStreamedText,
    ...attachmentSession,
    chooseFiles,
    send: task.send,
    cancel: task.cancel,
    resume: task.resume,
    decidePermission: task.decidePermission,
    decideApproval: task.decideApproval,
    selectConversation,
    conversationQuery,
    showingArchived,
    scrollPositions,
    rememberScroll,
    refreshConversations,
    loadOlder,
    ...conversationActions,
  };
}
