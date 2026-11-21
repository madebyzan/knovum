<script setup lang="ts">
import { ScanText } from "@lucide/vue";
import { useI18n } from "vue-i18n";

import SpaceTextQuote from "../../knowledge/presentation/SpaceTextQuote.vue";
import type { PresentationSlideView } from "../slides-deck-projection";

defineProps<{
  slide: PresentationSlideView;
  exactQuote?: string | undefined;
}>();

const { t } = useI18n();
</script>

<template>
  <div :data-slide-number="slide.slideNumber" class="space-y-3">
    <div v-for="block in slide.textBlocks" :key="block.chunkIndex" :data-chunk-index="block.chunkIndex">
      <SpaceTextQuote :text="block.text" :exact="exactQuote" />
    </div>
    <div v-if="slide.ocrPassages.length" class="space-y-1.5">
      <div
        data-reader-ignore-selection
        class="flex items-center gap-1.5 text-caption-sm font-medium text-fg-tertiary select-none"
      >
        <ScanText class="size-3" />
        <span>{{ t("spaces.inspector.recognizedText") }}</span>
      </div>
      <SpaceTextQuote
        v-for="(passage, idx) in slide.ocrPassages"
        :key="idx"
        :text="passage"
        :exact="exactQuote"
      />
    </div>
  </div>
</template>
