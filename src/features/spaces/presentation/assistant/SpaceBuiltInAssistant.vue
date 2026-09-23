<script setup lang="ts">
import { MoreHorizontal, Plus } from "@lucide/vue";
import { storeToRefs } from "pinia";
import { computed, shallowRef, watch } from "vue";
import { useI18n } from "vue-i18n";

import UiButton from "../../../../shared/ui/UiButton.vue";
import UiDropdownMenu from "../../../../shared/ui/UiDropdownMenu.vue";
import UiInput from "../../../../shared/ui/UiInput.vue";
import UiSelect from "../../../../shared/ui/UiSelect.vue";
import { useBuiltInAssistant } from "../../assistant/use-built-in-assistant";
import { useSpaceStore } from "../../session/space-store";
import SpaceAssistantComposer from "./SpaceAssistantComposer.vue";
import SpaceAssistantConversation from "./SpaceAssistantConversation.vue";

const store = useSpaceStore();
const { activeSpaceId } = storeToRefs(store);
const { t } = useI18n();
const {
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
  send,
  cancel,
  selectConversation,
  renameSelected,
  archiveSelected,
  deleteSelected,
  exportSelected,
  loadOlder,
  attachments,
  branchFrom,
  regenerate,
  conversationQuery,
  showingArchived,
  refreshConversations,
} = useBuiltInAssistant(activeSpaceId);
const renaming = shallowRef(false);
const titleDraft = shallowRef("");
const selectedConversation = computed(() =>
  conversations.value.find(({ id }) => id === selectedConversationId.value),
);
const conversationOptions = computed(() =>
  conversations.value.map(({ id, title }) => ({ value: id, label: title })),
);
const conversationActions = computed(() => [
  { label: t("spaces.assistant.renameConversation"), onSelect: () => (renaming.value = true) },
  { label: t("spaces.assistant.exportConversation"), onSelect: () => void exportSelected() },
  {
    label: t(
      showingArchived.value ? "spaces.assistant.restoreConversation" : "spaces.assistant.archiveConversation",
    ),
    onSelect: () => void archiveSelected(!showingArchived.value),
  },
  {
    label: t("spaces.assistant.deleteConversation"),
    destructive: true,
    onSelect: () => void deleteSelected(),
  },
]);

watch(selectedConversation, (conversation) => {
  titleDraft.value = conversation?.title ?? "";
  renaming.value = false;
});

async function saveTitle() {
  await renameSelected(titleDraft.value);
  renaming.value = false;
}

function toggleArchived() {
  showingArchived.value = !showingArchived.value;
  void refreshConversations();
}
</script>

<template>
  <section class="flex h-full min-h-0 flex-col bg-panel">
    <header class="shrink-0 space-y-2 p-2.5">
      <div class="flex items-end gap-1">
        <UiSelect
          v-model="selectedConversationId"
          class="min-w-0 flex-1"
          :label="$t('spaces.assistant.knowledgeThreads')"
          :options="conversationOptions"
          :placeholder="t('spaces.assistant.newKnowledgeThread')"
          :disabled="loading"
          @update:model-value="selectConversation"
        />
        <UiButton
          size="sm"
          variant="ghost"
          :aria-label="t('spaces.assistant.newKnowledgeThread')"
          :disabled="loading"
          @click="selectConversation('')"
        >
          <Plus class="size-(--knovum-icon-size-sm)" />
        </UiButton>
        <UiDropdownMenu v-if="selectedConversation" :items="conversationActions">
          <template #trigger>
            <UiButton size="sm" variant="ghost" :aria-label="$t('spaces.assistant.conversationActions')">
              <MoreHorizontal class="size-(--knovum-icon-size-sm)" />
            </UiButton>
          </template>
        </UiDropdownMenu>
      </div>
      <form v-if="renaming" class="flex gap-1" @submit.prevent="saveTitle">
        <UiInput v-model="titleDraft" class="min-w-0 flex-1" autofocus />
        <UiButton size="sm" variant="primary" type="submit">{{ $t("common.save") }}</UiButton>
      </form>
      <div class="flex gap-1">
        <UiInput
          v-model="conversationQuery"
          class="min-w-0 flex-1"
          :placeholder="t('spaces.assistant.searchConversations')"
          @update:model-value="refreshConversations"
        />
        <UiButton size="sm" :variant="showingArchived ? 'secondary' : 'ghost'" @click="toggleArchived">
          {{ t("spaces.assistant.archivedConversations") }}
        </UiButton>
      </div>
    </header>
    <div class="min-h-0 flex-1 border-t border-divider">
      <p
        v-if="attachments.length"
        class="border-b border-divider px-3 py-1.5 text-micro text-foreground-secondary"
      >
        {{ t("spaces.assistant.attachedResources", { count: attachments.length }) }}
      </p>
      <SpaceAssistantConversation
        class="h-full"
        :messages="messages"
        :busy="loading"
        :error="error"
        :streamed-text="streamedText"
        :has-older="hasOlder"
        :loading-older="loadingOlder"
        @load-older="loadOlder"
        @branch="branchFrom"
        @regenerate="regenerate"
      />
      <p
        v-if="taskState === 'waiting' || taskState === 'paused'"
        class="border-t border-divider px-3 py-2 text-caption text-warning"
      >
        Task {{ taskState }}. Resolve it in Run Center or cancel it here.
      </p>
    </div>
    <SpaceAssistantComposer
      v-model="prompt"
      v-model:mode="mode"
      v-model:target="target"
      v-model:max-turns="maxTurns"
      v-model:max-tool-calls="maxToolCalls"
      v-model:max-total-tokens="maxTotalTokens"
      v-model:max-duration-seconds="maxDurationSeconds"
      :busy="loading || Boolean(activeRunId)"
      @send="send"
      @cancel="cancel"
    />
  </section>
</template>
