<script setup lang="ts">
import { takeRight } from "es-toolkit";
import { computed } from "vue";
import { useI18n } from "vue-i18n";

import UiSpinner from "../../../../shared/ui/UiSpinner.vue";
import type { SpaceMediaAnalysisPreview } from "../analysis-preview";
import { formatDurationMs } from "../format-duration";

const props = defineProps<{ preview: SpaceMediaAnalysisPreview }>();
const { t } = useI18n();
const recent = computed(() => takeRight(props.preview.segments, 5));
</script>

<template>
  <section class="mt-2.5 overflow-hidden rounded-lg bg-surface-control/40 px-2.5 py-2">
    <header class="flex items-center gap-2 pb-1.5">
      <UiSpinner size="sm" />
      <span class="min-w-0 flex-1 text-caption font-medium text-fg-secondary">
        {{ t("spaces.transcript.text.live.label") }}
      </span>
      <span class="text-caption-sm text-fg-tertiary tabular-nums">{{ preview.progress }}%</span>
    </header>
    <div class="max-h-52 space-y-1 overflow-hidden">
      <div v-for="segment in recent" :key="`${segment.startMs}:${segment.endMs}`" class="flex gap-2 py-0.5">
        <span class="w-10 shrink-0 pt-0.5 font-mono text-caption-sm text-fg-tertiary tabular-nums">
          {{ formatDurationMs(segment.startMs) }}
        </span>
        <p class="line-clamp-2 min-w-0 flex-1 text-caption leading-5 text-fg-secondary">
          {{ segment.text }}
        </p>
      </div>
    </div>
    <footer class="mt-1.5 border-t border-stroke-divider/60 pt-1.5 text-caption-sm text-fg-tertiary">
      {{ t("spaces.transcript.text.live.hint", { language: preview.language }) }}
    </footer>
  </section>
</template>
