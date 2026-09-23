import { computed, onMounted, shallowRef, watch, type Ref } from "vue";

import type { ExecutionTarget } from "../../../foundation/ai/ai-models";
import { useCanvasEditorSession } from "../canvas/editor-session";
import { useImageCommentSession } from "../comments/image-comment-session";
import { useSpaceStore } from "../session/space-store";
import {
  cancelAssistantTask,
  listAssistantConversations,
  listAssistantMessages,
  startAssistantTask,
  type AssistantConversation,
  type AssistantMessage,
  type AssistantMode,
  type StartAssistantTaskInput,
} from "./assistant-client";
import { useAssistantConversationActions } from "./use-assistant-conversation-actions";

export function useBuiltInAssistant(spaceId: Ref<string | undefined>) {
  const store = useSpaceStore();
  const canvas = useCanvasEditorSession();
  const imageComments = useImageCommentSession();
  const conversations = shallowRef<AssistantConversation[]>([]);
  const messages = shallowRef<AssistantMessage[]>([]);
  const selectedConversationId = shallowRef("");
  const prompt = shallowRef("");
  watch(
    imageComments.conversationDraft,
    (draft) => {
      if (!draft || !spaceId.value || !store.nodes.some(({ id }) => id === draft.nodeId)) return;
      prompt.value = [prompt.value.trim(), draft.text].filter(Boolean).join("\n\n");
      imageComments.takeConversationDraft();
    },
    { immediate: true },
  );
  const mode = shallowRef<AssistantMode>("ask");
  const target = shallowRef<ExecutionTarget>();
  const maxTurns = shallowRef("12");
  const maxToolCalls = shallowRef("32");
  const maxTotalTokens = shallowRef("100000");
  const maxDurationSeconds = shallowRef("900");
  const activeRunId = shallowRef<string>();
  const taskState = shallowRef<string>();
  const loading = shallowRef(false);
  const loadingOlder = shallowRef(false);
  const hasOlder = shallowRef(false);
  const error = shallowRef<string>();
  const streamedText = shallowRef("");
  const conversationQuery = shallowRef("");
  const showingArchived = shallowRef(false);
  const selectedConversation = computed(() =>
    conversations.value.find(({ id }) => id === selectedConversationId.value),
  );
  const attachments = computed(() => {
    const selected = new Set(canvas.selection.value.nodeIds);
    return store.nodes.flatMap((node) => {
      if (!selected.has(node.id)) return [];
      const revision = node.data.knowledgeRevision ?? node.data.sourceRevision;
      return revision ? [{ nodeId: node.id, sourceRevision: revision, role: "source" }] : [];
    });
  });

  function pendingTask(): StartAssistantTaskInput | undefined {
    const selected = selectedConversation.value;
    const text = prompt.value.trim();
    if (!spaceId.value || !target.value || !text || loading.value) return;
    return {
      spaceId: spaceId.value,
      ...(selected ? { conversationId: selected.id } : {}),
      ...(selected?.branchId ? { branchId: selected.branchId } : {}),
      prompt: text,
      mode: mode.value,
      target: target.value,
      maxTurns: Number(maxTurns.value),
      maxToolCalls: Number(maxToolCalls.value),
      maxTotalTokens: Number(maxTotalTokens.value),
      maxDurationSeconds: Number(maxDurationSeconds.value),
      attachments: attachments.value,
    };
  }

  async function refreshConversations() {
    if (!spaceId.value) return;
    conversations.value = await listAssistantConversations(
      spaceId.value,
      conversationQuery.value,
      showingArchived.value,
    );
  }

  async function selectConversation(id: string) {
    selectedConversationId.value = id;
    if (!id) {
      messages.value = [];
      return;
    }
    const conversation = conversations.value.find((item) => item.id === id);
    if (!conversation?.branchId) return;
    mode.value = conversation.mode;
    const page = await listAssistantMessages(id, conversation.branchId);
    messages.value = page;
    hasOlder.value = page.length === 100;
  }

  async function loadOlder() {
    const conversation = selectedConversation.value;
    const first = messages.value[0];
    if (!conversation?.branchId || !first || loadingOlder.value || !hasOlder.value) return;
    loadingOlder.value = true;
    try {
      const page = await listAssistantMessages(conversation.id, conversation.branchId, first);
      messages.value = [...page, ...messages.value];
      hasOlder.value = page.length === 100;
    } finally {
      loadingOlder.value = false;
    }
  }

  async function send() {
    const request = pendingTask();
    if (!request) return;
    loading.value = true;
    error.value = undefined;
    prompt.value = "";
    try {
      streamedText.value = "";
      const task = await startAssistantTask(request, (delta) => (streamedText.value += delta));
      const result = task.accepted;
      activeRunId.value = result.runId;
      const state = await task.settled;
      taskState.value = state;
      streamedText.value = "";
      if (state !== "succeeded" && state !== "paused") {
        throw new Error(`Assistant task ended as ${state}`);
      }
      await refreshConversations();
      await selectConversation(result.conversationId);
    } catch (reason) {
      error.value = String(reason);
    } finally {
      loading.value = false;
      if (isTerminalTaskState(taskState.value)) {
        activeRunId.value = undefined;
      }
    }
  }

  async function cancel() {
    if (activeRunId.value) await cancelAssistantTask(activeRunId.value);
  }

  watch(spaceId, async () => {
    selectedConversationId.value = "";
    messages.value = [];
    await refreshConversations();
  });
  onMounted(refreshConversations);
  const conversationActions = useAssistantConversationActions({
    selected: selectedConversation,
    selectedId: selectedConversationId,
    messages,
    loading,
    prompt,
    refresh: refreshConversations,
    select: selectConversation,
    send,
  });

  return {
    conversations,
    messages,
    selectedConversationId,
    prompt,
    mode,
    target,
    maxTurns,
    maxToolCalls,
    maxTotalTokens,
    maxDurationSeconds,
    activeRunId,
    taskState,
    loading,
    loadingOlder,
    hasOlder,
    error,
    streamedText,
    attachments,
    send,
    cancel,
    selectConversation,
    conversationQuery,
    showingArchived,
    refreshConversations,
    loadOlder,
    ...conversationActions,
  };
}

function isTerminalTaskState(state: string | undefined) {
  return !state || new Set(["succeeded", "completed_with_errors", "failed", "cancelled"]).has(state);
}
