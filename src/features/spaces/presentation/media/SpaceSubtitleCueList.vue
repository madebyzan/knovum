<script setup lang="ts">
import { useVirtualizer } from "@tanstack/vue-virtual";
import { computed, ref } from "vue";

import { formatDurationMs } from "../../media/format-duration";
import type { SpaceSubtitleCue } from "../../media/knowledge-schema";

const props = defineProps<{ cues: readonly SpaceSubtitleCue[]; activeIndex: number }>();
const emit = defineEmits<{ select: [index: number] }>();
const scroller = ref<HTMLDivElement | null>(null);
const virtualizer = useVirtualizer(
  computed(() => ({
    count: props.cues.length,
    getScrollElement: () => scroller.value,
    estimateSize: () => 52,
    overscan: 8,
  })),
);
const rows = computed(() => virtualizer.value.getVirtualItems());
const totalSize = computed(() => virtualizer.value.getTotalSize());
</script>

<template>
  <div ref="scroller" class="h-64 overflow-y-auto rounded-panel border border-divider bg-surface/45">
    <div class="relative w-full" :style="{ height: `${totalSize}px` }">
      <button
        v-for="row in rows"
        :key="String(row.key)"
        type="button"
        class="absolute top-0 left-0 flex w-full items-start gap-2 border-b border-divider px-2 py-2 text-left hover:bg-control-hover"
        :class="row.index === activeIndex && 'bg-selected'"
        :style="{ height: `${row.size}px`, transform: `translateY(${row.start}px)` }"
        @click="emit('select', row.index)"
      >
        <span class="w-10 shrink-0 text-micro text-foreground-tertiary">{{
          formatDurationMs(cues[row.index]!.startMs)
        }}</span>
        <span class="line-clamp-2 text-caption text-foreground-secondary">{{ cues[row.index]!.text }}</span>
      </button>
    </div>
  </div>
</template>
