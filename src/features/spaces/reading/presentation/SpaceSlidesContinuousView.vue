<script setup lang="ts">
import { useTemplateRef } from "vue";

import UiScrollEdgeFade from "../../../../shared/ui/UiScrollEdgeFade.vue";
import type { PresentationSlideView } from "../slides-deck-projection";
import SpaceSlidesNotesSection from "./SpaceSlidesNotesSection.vue";

defineProps<{
  slides: readonly PresentationSlideView[];
  showNotes: boolean;
  targetSlide?: number | undefined;
  quote?: string | undefined;
}>();

const emit = defineEmits<{ hoverSlide: [slideNumber: number] }>();
const scroller = useTemplateRef<HTMLDivElement>("scroller");

function scrollToSlide(slideNumber: number) {
  scroller.value
    ?.querySelector<HTMLElement>(`[data-slide-number="${slideNumber}"]`)
    ?.scrollIntoView({ behavior: "smooth", block: "start" });
}

defineExpose({ scrollToSlide });
</script>

<template>
  <div class="relative flex min-h-0 flex-1 overflow-hidden">
    <div ref="scroller" class="min-h-0 flex-1 space-y-3 overflow-y-auto bg-surface-muted px-4 pt-4 pb-10">
      <section
        v-for="(slide, idx) in slides"
        :key="slide.slideNumber"
        :data-slide-number="slide.slideNumber"
        class="relative scroll-mt-3 overflow-hidden rounded-[4px] border border-stroke-divider/80 bg-surface-raised shadow-panel"
        @mouseenter="emit('hoverSlide', slide.slideNumber)"
      >
        <img
          v-if="slide.images[0]"
          :src="slide.images[0].src"
          :alt="`Slide ${idx + 1}`"
          class="block w-full bg-white object-contain"
          loading="lazy"
        />
        <span
          data-reader-ignore-selection
          class="pointer-events-none absolute top-2 right-2 rounded-sm bg-black/55 px-1.5 py-0.5 text-[11px] font-medium text-white tabular-nums select-none"
        >
          {{ idx + 1 }} / {{ slides.length }}
        </span>
        <div
          v-if="(!slide.images[0] || showNotes) && (slide.textBlocks.length || slide.ocrPassages.length)"
          class="border-t border-stroke-divider/70 px-5 py-4"
        >
          <SpaceSlidesNotesSection
            :slide="slide"
            :exact-quote="slide.slideNumber === targetSlide ? quote : undefined"
          />
        </div>
      </section>
    </div>
    <UiScrollEdgeFade surface="muted" top-class="h-5" bottom-class="h-9" />
  </div>
</template>
