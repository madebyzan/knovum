<script setup lang="ts">
import { LoaderCircle } from "@lucide/vue";
import { takeRight } from "es-toolkit";
import { computed } from "vue";
import { useI18n } from "vue-i18n";

import type { SpaceMediaAnalysisPreview } from "../../media/analysis-preview";
import { formatDurationMs } from "../../media/format-duration";

const props = defineProps<{ preview: SpaceMediaAnalysisPreview }>();
const { t } = useI18n();
const recent = computed(() => takeRight(props.preview.segments, 5));
</script>

<template>
  <section class="mt-3 overflow-hidden rounded-panel border border-stroke-divider bg-surface-muted/45">
    <header class="flex items-center gap-2 border-b border-stroke-divider px-2.5 py-2">
      <LoaderCircle class="size-3 animate-spin text-fg-accent" :stroke-width="1.7" />
      <span class="min-w-0 flex-1 text-caption font-medium text-fg-secondary">
        {{ t("spaces.transcript.text.live") }}
      </span>
      <span class="text-caption-sm text-fg-tertiary tabular-nums">{{ preview.progress }}%</span>
    </header>
    <div class="max-h-52 overflow-hidden">
      <div
        v-for="segment in recent"
        :key="`${segment.startMs}:${segment.endMs}`"
        class="flex gap-2.5 border-b border-stroke-divider/60 px-2.5 py-2 last:border-b-0"
      >
        <span class="w-10 shrink-0 pt-0.5 text-caption-sm text-fg-tertiary tabular-nums">
          {{ formatDurationMs(segment.startMs) }}
        </span>
        <p class="line-clamp-2 min-w-0 flex-1 text-caption leading-relaxed text-fg-secondary">
          {{ segment.text }}
        </p>
      </div>
    </div>
    <footer class="border-t border-stroke-divider px-2.5 py-1.5 text-caption-sm text-fg-tertiary">
      {{ t("spaces.transcript.text.liveHint", { language: preview.language }) }}
    </footer>
  </section>
</template>
