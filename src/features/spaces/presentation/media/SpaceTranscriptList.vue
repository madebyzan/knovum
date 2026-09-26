<script setup lang="ts">
import { useVirtualizer } from "@tanstack/vue-virtual";
import { search } from "fast-fuzzy";
import { computed, nextTick, ref, watch, type ComponentPublicInstance } from "vue";
import { useI18n } from "vue-i18n";

import UiSearchInput from "../../../../shared/ui/UiSearchInput.vue";
import UiSwitch from "../../../../shared/ui/UiSwitch.vue";
import type { SpaceTranscriptEdit, SpaceTranscriptSegment } from "../../media/schema";
import SpaceTranscriptRow from "./SpaceTranscriptRow.vue";

const props = defineProps<{
  nodeId: string;
  segments: SpaceTranscriptSegment[];
  currentMs?: number;
  disabled?: boolean;
}>();
const emit = defineEmits<{
  seek: [positionMs: number];
  save: [payload: SpaceTranscriptEdit];
}>();
const { t } = useI18n();
const scroller = ref<HTMLDivElement | null>(null);
const query = ref("");
const autoFollow = ref(true);
const visibleIndices = computed(() => {
  const needle = query.value.trim();
  if (!needle) return props.segments.map((_, index) => index);
  const indexed = props.segments.map((segment, index) => ({ segment, index }));
  const matched = search(needle, indexed, {
    keySelector: ({ segment }) => segment.text,
  });
  return matched.map((match) => match.index);
});
const activeIndex = computed(() => {
  const current = props.currentMs;
  if (current === undefined) return -1;
  return props.segments.findLastIndex((segment) => segment.startMs <= current);
});
const activeVisibleIndex = computed(() => visibleIndices.value.indexOf(activeIndex.value));
const virtualizer = useVirtualizer(
  computed(() => ({
    count: visibleIndices.value.length,
    getScrollElement: () => scroller.value,
    estimateSize: () => 44,
    overscan: 8,
  })),
);
const rows = computed(() => virtualizer.value.getVirtualItems());
const totalSize = computed(() => virtualizer.value.getTotalSize());
watch(activeVisibleIndex, (index) => {
  if (autoFollow.value && index >= 0) virtualizer.value.scrollToIndex(index, { align: "center" });
});
watch(
  () => props.segments,
  () => virtualizer.value.measure(),
  { deep: true },
);
function measureElement(element: Element | ComponentPublicInstance | null) {
  const row = element instanceof Element ? element : element?.$el;
  if (row instanceof Element) virtualizer.value.measureElement(row);
}
async function remeasure() {
  await nextTick();
  requestAnimationFrame(() => virtualizer.value.measure());
}
</script>

<template>
  <div class="overflow-hidden rounded-panel border border-divider bg-surface/45">
    <div class="flex items-center gap-2 border-b border-divider p-2">
      <UiSearchInput
        v-model="query"
        class="min-w-0 flex-1"
        :placeholder="t('spaces.transcript.text.search')"
      />
      <label class="flex shrink-0 items-center gap-1.5 text-micro text-foreground-tertiary">
        {{ t("spaces.media.player.followPlayback") }}
        <UiSwitch v-model="autoFollow" :label="t('spaces.media.player.followPlayback')" />
      </label>
    </div>
    <div ref="scroller" class="h-80 overflow-y-auto">
      <div class="relative w-full" :style="{ height: `${totalSize}px` }">
        <SpaceTranscriptRow
          v-for="row in rows"
          :key="visibleIndices[row.index]!"
          :node-id="nodeId"
          :ref="measureElement"
          :data-index="row.index"
          class="absolute top-0 left-0 w-full"
          :style="{ transform: `translateY(${row.start}px)` }"
          :segment="segments[visibleIndices[row.index]!]!"
          :index="visibleIndices[row.index]!"
          :active="visibleIndices[row.index] === activeIndex"
          v-bind="disabled === undefined ? {} : { disabled }"
          @seek="emit('seek', $event)"
          @save="emit('save', $event)"
          @editing="remeasure"
        />
      </div>
    </div>
    <p v-if="!visibleIndices.length" class="p-4 text-center text-caption text-foreground-tertiary">
      {{ t("spaces.transcript.text.noMatches") }}
    </p>
  </div>
</template>
