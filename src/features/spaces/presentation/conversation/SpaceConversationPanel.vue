<script setup lang="ts">
import { MoreHorizontal, Plus } from "@lucide/vue";
import { storeToRefs } from "pinia";
import { computed, shallowRef, watch } from "vue";
import { useI18n } from "vue-i18n";

import UiButton from "../../../../shared/ui/UiButton.vue";
import UiDropdownMenu from "../../../../shared/ui/UiDropdownMenu.vue";
import UiIconButton from "../../../../shared/ui/UiIconButton.vue";
import UiInput from "../../../../shared/ui/UiInput.vue";
import { useConversationSessionStore } from "../../conversation/conversation-session-store";
import SpaceContextHeader from "../SpaceContextHeader.vue";
import SpaceConversationComposer from "./SpaceConversationComposer.vue";
import SpaceConversationHistory from "./SpaceConversationHistory.vue";
import SpaceConversationMessages from "./SpaceConversationMessages.vue";
import SpaceConversationRunStatus from "./SpaceConversationRunStatus.vue";

const { t } = useI18n();
const emit = defineEmits<{ close: [] }>();
const conversationSession = useConversationSessionStore();
const {
  conversations,
  messages,
  selectedConversationId,
  prompt,
  target,
  activeRunId,
  activeConversationId,
  pendingPermission,
  approvals,
  decidingApproval,
  visibleTaskState,
  visibleWaitReason,
  loading,
  loadingOlder,
  hasOlder,
  visibleError,
  visiblePermission,
  decidingPermission,
  visibleStreamedText,
  visibleAttachments,
  attachmentCandidates,
  conversationQuery,
  showingArchived,
  scrollPositions,
} = storeToRefs(conversationSession);
const historyOpen = shallowRef(false);
const renaming = shallowRef(false);
const titleDraft = shallowRef("");
const selectedConversation = computed(() =>
  conversations.value.find(({ id }) => id === selectedConversationId.value),
);
const conversationActions = computed(() => [
  { label: t("spaces.assistant.renameConversation"), onSelect: () => (renaming.value = true) },
  {
    label: t("spaces.assistant.exportConversation"),
    onSelect: () => void conversationSession.exportSelected(),
  },
  {
    label: t(
      showingArchived.value ? "spaces.assistant.restoreConversation" : "spaces.assistant.archiveConversation",
    ),
    onSelect: () => void conversationSession.archiveSelected(!showingArchived.value),
  },
  {
    label: t("spaces.assistant.deleteConversation"),
    destructive: true,
    onSelect: () => void conversationSession.deleteSelected(),
  },
]);

watch(selectedConversation, (conversation) => {
  titleDraft.value = conversation?.title ?? "";
  renaming.value = false;
});

async function saveTitle() {
  await conversationSession.renameSelected(titleDraft.value);
  renaming.value = false;
}
</script>

<template>
  <section class="flex h-full min-h-0 min-w-0 flex-col bg-panel">
    <SpaceContextHeader
      :title="selectedConversation?.title || t('spaces.assistant.title')"
      @close="emit('close')"
    >
      <template #actions>
        <SpaceConversationHistory
          v-model:open="historyOpen"
          v-model:query="conversationQuery"
          v-model:archived="showingArchived"
          :conversations="conversations"
          :selected-id="selectedConversationId"
          @select="conversationSession.selectConversation"
          @refresh="conversationSession.refreshConversations"
        />
        <UiIconButton
          :label="t('spaces.assistant.newKnowledgeThread')"
          size="sm"
          :disabled="loading"
          @click="conversationSession.selectConversation('')"
        >
          <Plus class="size-(--knovum-icon-size-sm)" />
        </UiIconButton>
        <UiDropdownMenu v-if="selectedConversation" :items="conversationActions">
          <template #trigger>
            <UiIconButton :label="t('spaces.assistant.conversationActions')" size="sm">
              <MoreHorizontal class="size-(--knovum-icon-size-sm)" />
            </UiIconButton>
          </template>
        </UiDropdownMenu>
      </template>
    </SpaceContextHeader>
    <form v-if="renaming" class="flex shrink-0 gap-1 border-b border-divider p-2" @submit.prevent="saveTitle">
      <UiInput v-model="titleDraft" class="min-w-0 flex-1" autofocus />
      <UiButton size="sm" variant="primary" type="submit">{{ t("common.save") }}</UiButton>
    </form>
    <button
      v-if="(pendingPermission || approvals.length) && activeConversationId !== selectedConversationId"
      type="button"
      class="bg-warning-subtle shrink-0 border-b border-warning/40 px-3 py-2 text-left text-caption text-foreground hover:bg-warning/15"
      @click="conversationSession.selectConversation(activeConversationId)"
    >
      {{ t("spaces.assistant.returnToPermission") }}
    </button>
    <button
      v-else-if="activeRunId && activeConversationId && activeConversationId !== selectedConversationId"
      type="button"
      class="shrink-0 border-b border-divider bg-control px-3 py-2 text-left text-caption text-foreground hover:bg-control-hover"
      @click="conversationSession.selectConversation(activeConversationId)"
    >
      {{ t("spaces.assistant.returnToRunningConversation") }}
    </button>
    <div class="min-h-0 min-w-0 flex-1">
      <SpaceConversationMessages
        class="h-full"
        :scroll-key="selectedConversationId"
        :saved-scroll-top="scrollPositions[selectedConversationId] ?? 0"
        :messages="messages"
        :busy="loading"
        :error="visibleError"
        :streamed-text="visibleStreamedText"
        :has-older="hasOlder"
        :loading-older="loadingOlder"
        :permission="visiblePermission"
        :deciding-permission="decidingPermission"
        :approvals="activeConversationId === selectedConversationId ? approvals : []"
        :deciding-approval="decidingApproval"
        @load-older="conversationSession.loadOlder"
        @branch="conversationSession.branchFrom"
        @regenerate="conversationSession.regenerate"
        @decide-permission="conversationSession.decidePermission"
        @decide-approval="conversationSession.decideApproval"
        @scroll-position="conversationSession.rememberScroll"
      />
    </div>
    <SpaceConversationRunStatus
      :state="visibleTaskState"
      :wait-reason="visibleWaitReason"
      :has-pending-decision="Boolean(visiblePermission || approvals.length)"
      @resume="conversationSession.resume"
    />
    <SpaceConversationComposer
      v-model="prompt"
      v-model:target="target"
      :attachments="visibleAttachments"
      :attachment-candidates="attachmentCandidates"
      :busy="loading || Boolean(activeRunId)"
      @add-selection="conversationSession.addSelectedAttachments"
      @add-attachment="(id) => conversationSession.addNodeAttachments([id])"
      @remove-attachment="conversationSession.removeAttachment"
      @send="conversationSession.send"
      @cancel="conversationSession.cancel"
    />
  </section>
</template>
