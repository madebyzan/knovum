<script setup lang="ts">
import { Plus, Search } from "@lucide/vue";
import { sortBy } from "es-toolkit";
import { search } from "fast-fuzzy";
import { computed, nextTick, onMounted, shallowRef, watch } from "vue";

import UiButton from "../../../../shared/ui/UiButton.vue";
import UiInput from "../../../../shared/ui/UiInput.vue";
import UiScrollArea from "../../../../shared/ui/UiScrollArea.vue";
import type { VideoTextOverlay } from "../../video-editing/project";
import { formatVideoTimelineTime } from "../../video-editing/timecode";

const props = defineProps<{
  items: VideoTextOverlay[];
  selectedTextId?: string | undefined;
}>();
defineEmits<{ select: [id: string]; add: [] }>();
const query = shallowRef("");
const selectedRow = shallowRef<HTMLElement | null>(null);
const visibleItems = computed(() => {
  const needle = query.value.trim();
  const matched = needle ? search(needle, props.items, { keySelector: ({ text }) => text }) : props.items;
  return sortBy(matched, [({ startMs }) => startMs]);
});

async function revealSelection() {
  await nextTick();
  selectedRow.value?.scrollIntoView({ block: "nearest" });
}
watch(() => props.selectedTextId, revealSelection);
onMounted(revealSelection);
</script>

<template>
  <div class="flex min-h-0 flex-1 flex-col">
    <div class="space-y-2 border-b border-stroke-divider p-3">
      <label class="sr-only" for="video-caption-search">{{
        $t("spaces.media.videoEditor.captions.search")
      }}</label>
      <div class="relative">
        <Search
          class="pointer-events-none absolute top-1/2 left-2.5 size-3.5 -translate-y-1/2 text-fg-tertiary"
        />
        <UiInput
          id="video-caption-search"
          v-model="query"
          class="pl-8"
          size="sm"
          :placeholder="$t('spaces.media.videoEditor.captions.search')"
        />
      </div>
      <UiButton class="w-full justify-center" size="sm" @click="$emit('add')">
        <Plus class="size-3.5" />{{ $t("spaces.media.videoEditor.text.add") }}
      </UiButton>
    </div>
    <UiScrollArea class="min-h-0 flex-1">
      <div class="space-y-1 p-2">
        <button
          v-for="item in visibleItems"
          :key="item.id"
          :ref="
            (value) => {
              if (item.id === selectedTextId) selectedRow = value as HTMLElement | null;
            }
          "
          type="button"
          class="w-full rounded-md px-3 py-2 text-left focus-visible:outline-2 focus-visible:outline-stroke-focus"
          :class="
            item.id === selectedTextId
              ? 'bg-surface-selected text-fg-accent'
              : 'text-fg-primary hover:bg-surface-control-hover'
          "
          :aria-pressed="item.id === selectedTextId"
          @click="$emit('select', item.id)"
        >
          <span
            class="block text-caption-sm tabular-nums"
            :class="item.id === selectedTextId ? 'text-fg-accent' : 'text-fg-tertiary'"
          >
            {{ formatVideoTimelineTime(item.startMs) }} – {{ formatVideoTimelineTime(item.endMs) }}
          </span>
          <span class="mt-0.5 block truncate text-caption-md">{{ item.text }}</span>
        </button>
        <p v-if="visibleItems.length === 0" class="px-3 py-4 text-caption-md text-fg-secondary">
          {{
            $t(
              query
                ? "spaces.media.videoEditor.captions.noMatches"
                : "spaces.media.videoEditor.captions.none",
            )
          }}
        </p>
      </div>
    </UiScrollArea>
  </div>
</template>
