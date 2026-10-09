<script setup lang="ts">
import { Check, History } from "@lucide/vue";
import { useDebounceFn } from "@vueuse/core";
import { computed } from "vue";
import { useI18n } from "vue-i18n";

import UiIconButton from "../../../../shared/ui/UiIconButton.vue";
import UiPopover from "../../../../shared/ui/UiPopover.vue";
import UiSearchInput from "../../../../shared/ui/UiSearchInput.vue";
import type { Conversation } from "../../contracts/generated/Conversation";

const open = defineModel<boolean>("open", { default: false });
const query = defineModel<string>("query", { required: true });
const archived = defineModel<boolean>("archived", { required: true });
const props = defineProps<{
  conversations: readonly Conversation[];
  selectedId: string;
}>();
const { locale, t } = useI18n();
const timeFormatter = computed(
  () => new Intl.DateTimeFormat(locale.value, { hour: "2-digit", minute: "2-digit" }),
);
const dateFormatter = computed(
  () => new Intl.DateTimeFormat(locale.value, { year: "numeric", month: "long", day: "numeric" }),
);
const groups = computed(() => {
  const today = new Date();
  const yesterday = new Date(today);
  yesterday.setDate(today.getDate() - 1);
  const todayKey = today.toDateString();
  const yesterdayKey = yesterday.toDateString();
  const groupsByDate = new Map<string, { key: string; label: string; items: Conversation[] }>();
  for (const conversation of props.conversations) {
    const date = new Date(conversation.updatedAt);
    const key = date.toDateString();
    const label =
      key === todayKey
        ? t("spaces.conversation.conversations.today")
        : key === yesterdayKey
          ? t("spaces.conversation.conversations.yesterday")
          : dateFormatter.value.format(date);
    if (!groupsByDate.has(key)) groupsByDate.set(key, { key, label, items: [] });
    groupsByDate.get(key)?.items.push(conversation);
  }
  return [...groupsByDate.values()];
});
const emit = defineEmits<{
  select: [conversationId: string];
  refresh: [];
}>();
const refreshSearch = useDebounceFn(() => emit("refresh"), 180);

function choose(conversationId: string) {
  emit("select", conversationId);
  open.value = false;
}

function toggleArchived() {
  archived.value = !archived.value;
  emit("refresh");
}
</script>

<template>
  <UiPopover v-model:open="open" side="bottom" align="end" class="w-80 max-w-[calc(100vw-24px)] p-0">
    <template #trigger>
      <UiIconButton
        :label="$t('spaces.conversation.conversations.history')"
        :title="$t('spaces.conversation.conversations.history')"
        size="sm"
      >
        <History class="size-3.5" />
      </UiIconButton>
    </template>
    <div class="px-3 pt-3 pb-2">
      <h2 class="mb-2 text-title-sm font-semibold text-fg-primary">
        {{
          archived
            ? t("spaces.conversation.conversations.archived")
            : t("spaces.conversation.conversations.history")
        }}
      </h2>
      <UiSearchInput
        v-model="query"
        autofocus
        size="sm"
        :label="t('spaces.conversation.conversations.search')"
        :placeholder="t('spaces.conversation.conversations.search')"
        @update:model-value="refreshSearch()"
      />
    </div>
    <div class="max-h-80 min-h-0 overflow-y-auto px-2 pb-2">
      <div v-for="group in groups" :key="group.key" class="mt-1">
        <p class="px-2 py-1.5 text-caption-sm text-fg-tertiary">{{ group.label }}</p>
        <button
          v-for="conversation in group.items"
          :key="conversation.id"
          type="button"
          class="flex w-full min-w-0 items-center gap-2 rounded-md px-2 py-2 text-left text-caption transition-colors duration-100 hover:bg-surface-control-hover focus-visible:outline-2 focus-visible:outline-stroke-focus"
          :class="
            conversation.id === selectedId ? 'bg-surface-accent-subtle text-fg-primary' : 'text-fg-secondary'
          "
          :aria-current="conversation.id === selectedId ? 'true' : undefined"
          @click="choose(conversation.id)"
        >
          <span class="min-w-0 flex-1 truncate">{{ conversation.title }}</span>
          <time
            class="shrink-0 text-caption-sm text-fg-tertiary"
            :datetime="new Date(conversation.updatedAt).toISOString()"
          >
            {{ timeFormatter.format(conversation.updatedAt) }}
          </time>
          <Check v-if="conversation.id === selectedId" class="size-3.5 shrink-0 text-fg-accent" />
        </button>
      </div>
      <div
        v-if="!conversations.length"
        class="flex items-center justify-center gap-2 px-2 py-5 text-caption text-fg-tertiary"
      >
        <History class="size-4" :stroke-width="1.5" />
        {{ t("spaces.conversation.conversations.none") }}
      </div>
    </div>
    <button
      type="button"
      class="w-full border-t border-stroke-divider px-3 py-2.5 text-center text-caption text-fg-accent transition-colors duration-100 hover:bg-surface-control-hover focus-visible:outline-2 focus-visible:outline-stroke-focus"
      @click="toggleArchived"
    >
      {{
        archived
          ? t("spaces.conversation.conversations.backToRecent")
          : t("spaces.conversation.conversations.viewArchived")
      }}
    </button>
  </UiPopover>
</template>
