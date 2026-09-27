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
import SpaceConversationHistory from "./SpaceConversationHistory.vue";

const { t } = useI18n();
const session = useConversationSessionStore();
const { conversations, selectedConversationId, loading, conversationQuery, showingArchived } =
  storeToRefs(session);
const selectedConversation = computed(() =>
  conversations.value.find(({ id }) => id === selectedConversationId.value),
);
const historyOpen = shallowRef(false);
const renaming = shallowRef(false);
const titleDraft = shallowRef("");
const actions = computed(() => [
  { label: t("common.rename"), onSelect: () => (renaming.value = true) },
  { label: t("spaces.assistant.conversations.export"), onSelect: () => void session.exportSelected() },
  {
    label: t(
      showingArchived.value
        ? "spaces.assistant.conversations.restore"
        : "spaces.assistant.conversations.archive",
    ),
    onSelect: () => void (showingArchived.value ? session.unarchiveSelected() : session.archiveSelected()),
  },
  {
    label: t("spaces.assistant.conversations.delete"),
    destructive: true,
    onSelect: () => void session.deleteSelected(),
  },
]);

watch(selectedConversation, (conversation) => {
  titleDraft.value = conversation?.title ?? "";
  renaming.value = false;
});

async function saveTitle() {
  await session.renameSelected(titleDraft.value);
  renaming.value = false;
}
</script>

<template>
  <SpaceContextHeader :title="selectedConversation?.title || t('spaces.assistant.title')">
    <template #actions>
      <SpaceConversationHistory
        v-model:open="historyOpen"
        v-model:query="conversationQuery"
        v-model:archived="showingArchived"
        :conversations="conversations"
        :selected-id="selectedConversationId"
        @select="session.selectConversation"
        @refresh="session.refreshConversations"
      />
      <UiIconButton
        :label="t('spaces.assistant.conversations.create')"
        size="sm"
        :disabled="loading"
        @click="session.selectConversation('')"
      >
        <Plus class="size-(--knovum-icon-size-sm)" />
      </UiIconButton>
      <UiDropdownMenu v-if="selectedConversation" :items="actions">
        <template #trigger>
          <UiIconButton :label="t('spaces.assistant.conversations.actions')" size="sm">
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
</template>
