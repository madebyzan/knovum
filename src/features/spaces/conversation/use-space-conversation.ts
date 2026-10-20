import { computed, shallowRef, watch, type Ref } from "vue";

import type { TaskTarget } from "../../../shared/contracts/generated/TaskTarget";
import type { Conversation } from "../contracts/generated/Conversation";
import type { ConversationMessage } from "../contracts/generated/ConversationMessage";
import { useSpaceStore } from "../session/space-store";
import { createConversationActions } from "./conversation-actions";
import { createConversationFilePicker } from "./conversation-file-reacquisition";
import { createConversationSelection } from "./conversation-selection";
import { useConversationActivities } from "./use-conversation-activities";
import { useConversationAttachments } from "./use-conversation-attachments";
import { useConversationBranches } from "./use-conversation-branches";
import { useConversationDraftPersistence } from "./use-conversation-draft-persistence";
import { useConversationExtensions } from "./use-conversation-extensions";
import { useConversationMessagePages } from "./use-conversation-message-pages";
import { useConversationSpaceRestoration } from "./use-conversation-space-restoration";
import { useConversationTask } from "./use-conversation-task";
import { useImageCommentDraft } from "./use-image-comment-draft";
export function useSpaceConversation(spaceId: Ref<string | undefined>) {
  const store = useSpaceStore();
  const attachmentSession = useConversationAttachments();
  const conversations = shallowRef<Conversation[]>([]);
  const selectedSnapshot = shallowRef<Conversation>();
  const messages = shallowRef<ConversationMessage[]>([]);
  const selectedConversationId = shallowRef("");
  const extensionSession = useConversationExtensions(spaceId, selectedConversationId);
  const prompt = shallowRef("");
  const target = shallowRef<TaskTarget>();
  const conversationQuery = shallowRef("");
  const showingArchived = shallowRef(false);
  const selectedConversation = computed(() =>
    selectedConversationId.value ? selectedSnapshot.value : undefined,
  );
  const pages = useConversationMessagePages({
    spaceId,
    selectedConversationId,
    selectedConversation,
    messages,
  });
  const {
    session: draft,
    error: draftError,
    reacquireFileNames,
  } = useConversationDraftPersistence({
    spaceId,
    conversationId: selectedConversationId,
    prompt,
    attachments: attachmentSession.attachments,
    extensions: extensionSession.extensionSelection,
  });
  const chooseFiles = createConversationFilePicker({
    spaceId,
    conversationId: selectedConversationId,
    ownerEpoch: draft.ownerEpoch,
    attachments: attachmentSession.attachments,
    reacquireFileNames,
    choose: attachmentSession.chooseFiles,
  });
  const { refreshConversations, selectConversation } = createConversationSelection({
    spaceId,
    query: conversationQuery,
    showingArchived,
    conversations,
    selectedId: selectedConversationId,
    selectedSnapshot,
    messages,
    hasOlder: pages.hasOlder,
    rememberDraft: draft.remember,
    restoreDraft: draft.restore,
    clearBranches: () => {
      branchSession.branches.value = [];
    },
    loadBranches: (conversationId) => branchSession.load(conversationId),
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
    prompt,
    target,
    attachments: attachmentSession.attachments,
    extensions: extensionSession.extensionSelection,
    hasUnresolvedFileReferences: computed(() => reacquireFileNames.value.length > 0),
    visibleAttachments: attachmentSession.visibleAttachments,
    messages,
    refresh: refreshConversations,
    select: selectConversation,
    moveDraft: draft.move,
    clearDraft: draft.clearSubmitted,
    restoreDraft: draft.restoreSubmitted,
  });
  const { error: restorationError } = useConversationSpaceRestoration({
    spaceId,
    reset: () => {
      selectedConversationId.value = "";
      selectedSnapshot.value = undefined;
      messages.value = [];
      branchSession.branches.value = [];
    },
    restoreDraft: (requestedSpace) => draft.restore("", requestedSpace),
    refresh: refreshConversations,
    restoreTask: task.restoreForSpace,
  });
  watch(
    () => store.askScopeRequest,
    async (request) => {
      if (!request) return;
      store.clearAskScopeRequest();
      const requestedSpace = spaceId.value;
      if (request.restart) await selectConversation("");
      if (spaceId.value !== requestedSpace) return;
      if (request.scope === "selection") attachmentSession.addNodeAttachments(request.nodeIds);
    },
    { immediate: true },
  );
  const conversationActions = createConversationActions({
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
    ...extensionSession,
    conversations,
    messages,
    branches: branchSession.branches,
    ...useConversationActivities(selectedConversationId, task.activeRunId, task.activityRevision),
    selectedConversationId,
    selectedConversation,
    prompt,
    draftError,
    restorationError,
    restoringDraft: draft.restoring,
    reacquireFileNames,
    target,
    activityRevision: task.activityRevision,
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
    discardFileReference: draft.discardFileReference,
    send: task.send,
    cancel: task.cancel,
    pause: task.pause,
    resume: task.resume,
    retryModel: task.retryModel,
    selectConversation,
    selectBranch: branchSession.selectBranch,
    conversationQuery,
    showingArchived,
    scrollTopByConversationId: pages.scrollTopByConversationId,
    rememberScroll: pages.rememberScroll,
    refreshConversations,
    loadOlder: pages.loadOlder,
    ...conversationActions,
  };
}
