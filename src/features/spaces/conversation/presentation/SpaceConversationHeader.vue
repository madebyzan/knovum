<script setup lang="ts">
import { GitBranch, MoreHorizontal, Pencil, Plus } from "@lucide/vue";
import { storeToRefs } from "pinia";
import { computed, nextTick, shallowRef, useTemplateRef, watch } from "vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";

import { getErrorMessage } from "../../../../foundation/errors/error-format";
import UiButton from "../../../../shared/ui/UiButton.vue";
import UiDropdownMenu from "../../../../shared/ui/UiDropdownMenu.vue";
import UiIconButton from "../../../../shared/ui/UiIconButton.vue";
import UiInput from "../../../../shared/ui/UiInput.vue";
import UiKnovumAiIcon from "../../../../shared/ui/UiKnovumAiIcon.vue";
import SpaceContextHeader from "../../workspace/presentation/SpaceContextHeader.vue";
import { useConversationSessionStore } from "../conversation-session-store";
import SpaceConversationHistory from "./SpaceConversationHistory.vue";

const { t } = useI18n();
const session = useConversationSessionStore();
const {
  conversations,
  branches,
  selectedConversationId,
  selectedConversation,
  loading,
  conversationQuery,
  showingArchived,
} = storeToRefs(session);
const historyOpen = shallowRef(false);
const renaming = shallowRef(false);
const titleDraft = shallowRef("");
const titleInput = useTemplateRef<InstanceType<typeof UiInput>>("titleInput");

async function beginRename() {
  if (!selectedConversation.value) return;
  titleDraft.value = selectedConversation.value.title;
  renaming.value = true;
  await nextTick();
  titleInput.value?.focus();
}

const actions = computed(() => [
  { label: t("common.rename"), onSelect: () => void beginRename() },
  { label: t("common.export"), onSelect: () => void session.exportSelected() },
  {
    label: t(showingArchived.value ? "common.restore" : "common.archive"),
    onSelect: () => void (showingArchived.value ? session.unarchiveSelected() : session.archiveSelected()),
  },
  {
    label: t("common.delete"),
    destructive: true,
    onSelect: () => void session.deleteSelected(),
  },
]);
const branchItems = computed(() =>
  branches.value.map((id, index) => ({
    label: t("spaces.conversation.conversations.branchNumber", { number: index + 1 }),
    checked: selectedConversation.value?.branchId === id,
    onSelect: () =>
      void session.selectBranch(id).catch((cause: unknown) => toast.error(getErrorMessage(cause))),
  })),
);

watch(selectedConversation, (conversation) => {
  titleDraft.value = conversation?.title ?? "";
  renaming.value = false;
});

async function saveTitle() {
  if (!renaming.value) return;
  await session.renameSelected(titleDraft.value);
  renaming.value = false;
}
</script>

<template>
  <SpaceContextHeader>
    <template #leading>
      <div class="flex min-w-0 flex-1 items-center gap-1.5">
        <UiKnovumAiIcon class="size-4 shrink-0 text-fg-secondary" />
        <UiInput
          v-if="renaming"
          ref="titleInput"
          v-model="titleDraft"
          size="sm"
          class="min-w-0 flex-1 text-label font-semibold"
          @blur="saveTitle"
          @commit.prevent="saveTitle"
          @cancel.prevent="renaming = false"
        />
        <button
          v-else-if="selectedConversation"
          type="button"
          class="group/title flex min-w-0 items-center gap-1 rounded-md px-1.5 py-0.5 text-left transition-colors hover:bg-surface-control-hover"
          :title="selectedConversation.title"
          @click="beginRename"
        >
          <h2 class="truncate text-label font-semibold tracking-title text-fg-primary">
            {{ selectedConversation.title }}
          </h2>
          <Pencil
            class="size-3 shrink-0 text-fg-tertiary opacity-0 transition-opacity duration-120 group-hover/title:opacity-100"
          />
        </button>
        <h2 v-else class="truncate text-label font-semibold tracking-title text-fg-primary">
          {{ t("spaces.conversation.title") }}
        </h2>
      </div>
    </template>
    <template #actions>
      <UiDropdownMenu v-if="branches.length > 1" :items="branchItems" align="end">
        <template #trigger>
          <UiButton size="sm" variant="ghost" :disabled="loading" class="gap-1 text-fg-secondary">
            <GitBranch class="size-3.5" />
            {{
              t("spaces.conversation.conversations.branchNumber", {
                number: branches.indexOf(selectedConversation?.branchId ?? "") + 1,
              })
            }}
          </UiButton>
        </template>
      </UiDropdownMenu>
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
        :label="t('spaces.conversation.conversations.create')"
        size="sm"
        tooltip
        :disabled="loading"
        @click="session.selectConversation('')"
      >
        <Plus class="size-3.5" :stroke-width="1.6" />
      </UiIconButton>
      <UiDropdownMenu v-if="selectedConversation" :items="actions">
        <template #trigger>
          <UiIconButton
            :label="t('spaces.conversation.conversations.actions')"
            :title="t('spaces.conversation.conversations.actions')"
            size="sm"
          >
            <MoreHorizontal class="size-3.5" :stroke-width="1.6" />
          </UiIconButton>
        </template>
      </UiDropdownMenu>
    </template>
  </SpaceContextHeader>
</template>
