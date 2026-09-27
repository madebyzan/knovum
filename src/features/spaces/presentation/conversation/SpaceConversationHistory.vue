<script setup lang="ts">
import { History } from "@lucide/vue";

import UiButton from "../../../../shared/ui/UiButton.vue";
import UiIconButton from "../../../../shared/ui/UiIconButton.vue";
import UiInput from "../../../../shared/ui/UiInput.vue";
import UiPopover from "../../../../shared/ui/UiPopover.vue";
import type { AssistantConversation } from "../../conversation/conversation-ipc";

const open = defineModel<boolean>("open", { default: false });
const query = defineModel<string>("query", { required: true });
const archived = defineModel<boolean>("archived", { required: true });
defineProps<{
  conversations: readonly AssistantConversation[];
  selectedId: string;
}>();
const emit = defineEmits<{
  select: [id: string];
  refresh: [];
}>();

function choose(id: string) {
  emit("select", id);
  open.value = false;
}

function toggleArchived() {
  archived.value = !archived.value;
  emit("refresh");
}
</script>

<template>
  <UiPopover v-model:open="open" side="bottom" align="end" class="w-80 max-w-[calc(100vw-24px)] p-2">
    <template #trigger>
      <UiIconButton :label="$t('spaces.assistant.conversations.history')" size="sm">
        <History class="size-3.5" />
      </UiIconButton>
    </template>
    <div class="flex min-w-0 items-center gap-2">
      <UiInput
        v-model="query"
        class="min-w-0 flex-1"
        :placeholder="$t('spaces.assistant.conversations.search')"
        @update:model-value="emit('refresh')"
      />
      <UiButton size="sm" :variant="archived ? 'secondary' : 'ghost'" @click="toggleArchived">
        {{ $t("spaces.assistant.conversations.archived") }}
      </UiButton>
    </div>
    <div class="mt-2 max-h-80 min-h-0 overflow-y-auto">
      <button
        v-for="conversation in conversations"
        :key="conversation.id"
        type="button"
        class="flex w-full min-w-0 rounded-md px-2.5 py-2 text-left text-label-md hover:bg-surface-control-hover focus-visible:outline-2 focus-visible:outline-stroke-focus"
        :class="conversation.id === selectedId ? 'bg-surface-control text-fg-primary' : 'text-fg-secondary'"
        @click="choose(conversation.id)"
      >
        <span class="truncate">{{ conversation.title }}</span>
      </button>
      <p v-if="!conversations.length" class="px-2 py-5 text-center text-caption-md text-fg-tertiary">
        {{ $t("spaces.assistant.conversations.none") }}
      </p>
    </div>
  </UiPopover>
</template>
