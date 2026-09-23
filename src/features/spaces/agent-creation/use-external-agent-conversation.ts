import { ask } from "@tauri-apps/plugin-dialog";
import { storeToRefs } from "pinia";
import { computed, onMounted, onUnmounted, shallowRef, watch } from "vue";
import { useI18n } from "vue-i18n";

import { useAiSettingsStore } from "../../../foundation/ai/ai-settings-store";
import { useSpaceStore } from "../session/space-store";
import {
  closeExternalConversation,
  deleteExternalConversation,
  listExternalConversations,
  listExternalMessages,
  sendExternalMessage,
  setExternalSessionMode,
  setExternalSessionOption,
  startExternalConversation,
  type ExternalConversationMessage,
  type ExternalConversationSummary,
} from "./external-conversation-client";
import { useAgentAttachments } from "./use-agent-attachments";
import { useAgentCreationCatalog } from "./use-agent-creation-catalog";

export function useExternalAgentConversation() {
  const { t } = useI18n();
  const { activeSpaceId } = storeToRefs(useSpaceStore());
  const aiSettings = useAiSettingsStore();
  const { agentOptions, refetch } = useAgentCreationCatalog();
  const conversations = shallowRef<ExternalConversationSummary[]>([]);
  const messages = shallowRef<ExternalConversationMessage[]>([]);
  const selectedId = shallowRef("");
  const prompt = shallowRef("");
  const busy = shallowRef(false);
  const error = shallowRef<string>();
  const attachmentState = useAgentAttachments();
  let refreshTimer: number | undefined;
  const selected = computed(() => conversations.value.find(({ id }) => id === selectedId.value));
  const conversationOptions = computed(() =>
    conversations.value.map((item) => ({ value: item.id, label: `${item.title} · ${item.state}` })),
  );

  async function refresh() {
    if (!activeSpaceId.value) return;
    conversations.value = await listExternalConversations(activeSpaceId.value);
    if (selectedId.value) messages.value = await listExternalMessages(selectedId.value);
  }
  async function select(id: string) {
    selectedId.value = id;
    messages.value = id ? await listExternalMessages(id) : [];
  }
  async function perform(operation: () => Promise<void>) {
    busy.value = true;
    error.value = undefined;
    try {
      await operation();
    } catch (reason) {
      error.value = String(reason);
    } finally {
      busy.value = false;
    }
  }
  async function start(input: Omit<Parameters<typeof startExternalConversation>[0], "spaceId">) {
    if (!activeSpaceId.value) return;
    await perform(async () => {
      const conversation = await startExternalConversation({ spaceId: activeSpaceId.value!, ...input });
      await refresh();
      await select(conversation.id);
    });
  }
  async function send() {
    const text = prompt.value.trim();
    if (!selected.value || selected.value.state !== "connected" || !text) return;
    await perform(async () => {
      await sendExternalMessage(selected.value!.id, text, attachmentState.paths.value);
      prompt.value = "";
      attachmentState.clear();
      await refresh();
    });
  }
  async function close() {
    if (!selected.value) return;
    await perform(async () => {
      await closeExternalConversation(selected.value!.id);
      await refresh();
    });
  }
  async function updateSession(operation: (id: string) => Promise<ExternalConversationSummary>) {
    if (!selected.value) return;
    await perform(async () => {
      const conversation = await operation(selected.value!.id);
      conversations.value = conversations.value.map((item) =>
        item.id === conversation.id ? conversation : item,
      );
    });
  }
  const setSessionOption = (optionId: string, value: string | boolean) =>
    updateSession((id) => setExternalSessionOption(id, optionId, value));
  const setSessionMode = (modeId: string) => updateSession((id) => setExternalSessionMode(id, modeId));
  async function remove() {
    if (!selected.value) return;
    const confirmed = await ask(t("spaces.assistant.deleteExternalConversationConfirm"), {
      kind: "warning",
      title: t("spaces.assistant.deleteExternalConversation"),
    });
    if (!confirmed) return;
    await perform(async () => {
      await deleteExternalConversation(selected.value!.id);
      selectedId.value = "";
      messages.value = [];
      await refresh();
    });
  }

  watch(activeSpaceId, refresh);
  onMounted(async () => {
    await aiSettings.ensureLoaded();
    await refetch();
    await refresh();
    refreshTimer = globalThis.setInterval(() => void refresh(), 1_000);
  });
  onUnmounted(() => globalThis.clearInterval(refreshTimer));
  return {
    agentOptions,
    attachmentState,
    busy,
    close,
    conversationOptions,
    error,
    messages,
    prompt,
    remove,
    select,
    selected,
    selectedId,
    send,
    setSessionMode,
    setSessionOption,
    start,
  };
}
