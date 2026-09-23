<script setup lang="ts">
import { Plus, Search } from "@lucide/vue";
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
const search = shallowRef("");
const selectedRow = shallowRef<HTMLElement | null>(null);
const visibleItems = computed(() => {
  const query = search.value.trim().toLocaleLowerCase();
  return props.items
    .filter((item) => !query || item.text.toLocaleLowerCase().includes(query))
    .sort((a, b) => a.startMs - b.startMs);
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
    <div class="space-y-2 border-b border-divider p-3">
      <label class="sr-only" for="video-caption-search">{{ $t("spaces.media.searchCaptions") }}</label>
      <div class="relative">
        <Search
          class="pointer-events-none absolute top-1/2 left-2.5 size-(--knovum-icon-size-sm) -translate-y-1/2 text-foreground-tertiary"
        />
        <UiInput
          id="video-caption-search"
          v-model="search"
          class="pl-8"
          size="sm"
          :placeholder="$t('spaces.media.searchCaptions')"
        />
      </div>
      <UiButton class="w-full justify-center" size="sm" @click="$emit('add')">
        <Plus class="size-(--knovum-icon-size-sm)" />{{ $t("spaces.media.addText") }}
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
          class="w-full rounded-control px-3 py-2 text-left focus-visible:outline-2 focus-visible:outline-focus-ring"
          :class="
            item.id === selectedTextId ? 'bg-selected text-accent' : 'text-foreground hover:bg-control-hover'
          "
          :aria-pressed="item.id === selectedTextId"
          @click="$emit('select', item.id)"
        >
          <span
            class="block text-micro tabular-nums"
            :class="item.id === selectedTextId ? 'text-accent' : 'text-foreground-tertiary'"
          >
            {{ formatVideoTimelineTime(item.startMs) }} – {{ formatVideoTimelineTime(item.endMs) }}
          </span>
          <span class="mt-0.5 block truncate text-caption">{{ item.text }}</span>
        </button>
        <p v-if="visibleItems.length === 0" class="px-3 py-4 text-caption text-foreground-secondary">
          {{ $t(search ? "spaces.media.noMatchingCaptions" : "spaces.media.noCaptions") }}
        </p>
      </div>
    </UiScrollArea>
  </div>
</template>
