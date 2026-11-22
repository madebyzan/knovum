<script setup lang="ts">
import { nextTick, useTemplateRef, watch } from "vue";

import type { PresentationSlideView } from "../slides-deck-projection";

const props = defineProps<{
  slides: readonly PresentationSlideView[];
  activeIndex: number;
}>();

const emit = defineEmits<{ select: [index: number] }>();
const stripRef = useTemplateRef<HTMLElement>("strip");

function onWheel(event: WheelEvent) {
  const strip = stripRef.value;
  if (!strip || Math.abs(event.deltaY) <= Math.abs(event.deltaX)) return;
  event.preventDefault();
  strip.scrollLeft += event.deltaY;
}

watch(
  () => props.activeIndex,
  async (index) => {
    const slide = props.slides[index];
    if (!slide) return;
    await nextTick();
    stripRef.value
      ?.querySelector<HTMLElement>(`[data-thumb-slide="${slide.slideNumber}"]`)
      ?.scrollIntoView({ behavior: "smooth", inline: "center", block: "nearest" });
  },
);
</script>

<template>
  <nav
    ref="strip"
    data-reader-ignore-selection
    class="flex shrink-0 items-center gap-2 overflow-x-auto border-t border-stroke-divider bg-surface-panel/90 px-3 py-2 select-none"
    @wheel="onWheel"
  >
    <button
      v-for="(slide, idx) in slides"
      :key="slide.slideNumber"
      :data-thumb-slide="slide.slideNumber"
      type="button"
      class="group relative aspect-video h-12 shrink-0 overflow-hidden rounded-[4px] border bg-surface-raised text-left transition-all"
      :class="
        idx === activeIndex
          ? 'border-surface-accent shadow-2xs ring-2 ring-surface-accent/40'
          : 'border-stroke-divider opacity-80 hover:border-stroke-strong hover:opacity-100'
      "
      @click="emit('select', idx)"
    >
      <img
        v-if="slide.images[0]"
        :src="slide.images[0].src"
        :alt="`Slide ${idx + 1}`"
        class="h-full w-full object-cover"
        loading="lazy"
      />
      <p v-else class="line-clamp-3 p-1 text-[8px] leading-tight text-fg-secondary">
        {{ slide.textBlocks[0]?.text ?? "" }}
      </p>
      <span
        class="absolute right-0.5 bottom-0.5 rounded-[2px] bg-black/65 px-1 text-[10px] leading-tight font-medium text-white tabular-nums"
      >
        {{ idx + 1 }}
      </span>
    </button>
  </nav>
</template>
