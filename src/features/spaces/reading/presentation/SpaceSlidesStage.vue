<script setup lang="ts">
import { ChevronLeft, ChevronRight } from "@lucide/vue";

import type { PresentationSlideView } from "../slides-deck-projection";
import SpaceSlidesNotesSection from "./SpaceSlidesNotesSection.vue";

defineProps<{
  slide: PresentationSlideView;
  canPrevious: boolean;
  canNext: boolean;
  showNotes: boolean;
  hasNotes: boolean;
  exactQuote?: string | undefined;
}>();

const emit = defineEmits<{ previous: []; next: [] }>();
</script>

<template>
  <div class="flex min-h-0 flex-1 flex-col overflow-hidden">
    <div
      :data-slide-number="slide.slideNumber"
      class="group/stage relative flex min-h-0 flex-1 items-center justify-center overflow-hidden bg-surface-muted p-4"
    >
      <button
        v-if="canPrevious"
        data-reader-ignore-selection
        type="button"
        class="absolute left-2 z-10 rounded-full border border-stroke-divider bg-surface-raised/90 p-1.5 text-fg-secondary opacity-0 shadow-xs transition-opacity group-hover/stage:opacity-100 hover:bg-surface-raised hover:text-fg-primary"
        @click="emit('previous')"
      >
        <ChevronLeft class="size-4" />
      </button>
      <img
        v-if="slide.images[0]"
        :src="slide.images[0].src"
        :alt="`Slide ${slide.slideNumber}`"
        class="max-h-full max-w-full rounded-[3px] border border-stroke-divider/70 bg-white object-contain shadow-sm"
      />
      <div
        v-else
        class="aspect-video max-h-full w-full overflow-y-auto rounded-[4px] border border-stroke-divider bg-surface-raised p-7 shadow-sm"
      >
        <SpaceSlidesNotesSection :slide="slide" :exact-quote="exactQuote" />
      </div>
      <button
        v-if="canNext"
        data-reader-ignore-selection
        type="button"
        class="absolute right-2 z-10 rounded-full border border-stroke-divider bg-surface-raised/90 p-1.5 text-fg-secondary opacity-0 shadow-xs transition-opacity group-hover/stage:opacity-100 hover:bg-surface-raised hover:text-fg-primary"
        @click="emit('next')"
      >
        <ChevronRight class="size-4" />
      </button>
    </div>
    <div
      v-if="showNotes && hasNotes"
      class="max-h-[38%] shrink-0 overflow-y-auto border-t border-stroke-divider bg-surface-raised px-6 py-4"
    >
      <SpaceSlidesNotesSection :slide="slide" :exact-quote="exactQuote" />
    </div>
  </div>
</template>
