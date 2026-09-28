<script setup lang="ts">
import { ChevronLeft, ChevronRight } from "@lucide/vue";
import { computed, shallowRef, watch } from "vue";

import UiPopover from "../../../../shared/ui/UiPopover.vue";
import SpaceConversationAttachment from "./SpaceConversationAttachment.vue";

interface Attachment {
  key: string;
  kind: "node" | "file";
  title: string;
  isImage: boolean;
  previewSource: string;
}

const props = defineProps<{ attachments: readonly Attachment[] }>();
const emit = defineEmits<{ remove: [key: string] }>();
const open = shallowRef(false);
const page = shallowRef(0);
const remaining = computed(() => props.attachments.slice(2));
const pageCount = computed(() => Math.ceil(remaining.value.length / 6));
const pageItems = computed(() => remaining.value.slice(page.value * 6, (page.value + 1) * 6));

watch(pageCount, (count) => {
  if (page.value >= count) page.value = Math.max(0, count - 1);
  if (!count) open.value = false;
});
watch(open, (value) => {
  if (!value) page.value = 0;
});
</script>

<template>
  <div v-if="attachments.length" class="flex min-w-0 flex-nowrap gap-2 px-3 pt-3">
    <SpaceConversationAttachment
      v-for="attachment in attachments.slice(0, 2)"
      :key="attachment.key"
      :attachment="attachment"
      @remove="emit('remove', $event)"
    />
    <UiPopover
      v-if="remaining.length"
      v-model:open="open"
      side="top"
      align="end"
      class="w-80 max-w-[calc(100vw-24px)] p-2"
    >
      <template #trigger>
        <button
          type="button"
          class="flex h-24 w-16 shrink-0 items-center justify-center rounded-lg border border-stroke-divider bg-surface-muted text-caption font-medium text-fg-secondary transition-colors hover:border-stroke-default-hover hover:bg-surface-control-hover hover:text-fg-primary focus-visible:outline-2 focus-visible:outline-stroke-focus"
          :aria-label="$t('spaces.assistant.attachments.expand', { count: remaining.length })"
        >
          +{{ remaining.length }}
        </button>
      </template>
      <div class="flex items-center justify-between px-1 pb-2">
        <span class="text-caption font-medium text-fg-primary">{{ $t("spaces.assistant.attachments.more") }}</span>
        <div v-if="pageCount > 1" class="flex items-center gap-1 text-caption-sm text-fg-tertiary">
          <button
            type="button"
            class="flex size-6 items-center justify-center rounded-md hover:bg-surface-control-hover disabled:opacity-40"
            :aria-label="$t('spaces.assistant.attachments.previousPage')"
            :disabled="page === 0"
            @click="page--"
          >
            <ChevronLeft class="size-3.5" />
          </button>
          {{ page + 1 }} / {{ pageCount }}
          <button
            type="button"
            class="flex size-6 items-center justify-center rounded-md hover:bg-surface-control-hover disabled:opacity-40"
            :aria-label="$t('spaces.assistant.attachments.nextPage')"
            :disabled="page === pageCount - 1"
            @click="page++"
          >
            <ChevronRight class="size-3.5" />
          </button>
        </div>
      </div>
      <div class="flex flex-wrap gap-2">
        <SpaceConversationAttachment
          v-for="attachment in pageItems"
          :key="attachment.key"
          :attachment="attachment"
          @remove="emit('remove', $event)"
        />
      </div>
    </UiPopover>
  </div>
</template>
