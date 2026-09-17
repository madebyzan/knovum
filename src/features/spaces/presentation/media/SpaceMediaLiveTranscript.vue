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
  <section class="mt-3 overflow-hidden rounded-panel border border-divider bg-surface/45">
    <header class="flex items-center gap-2 border-b border-divider px-2.5 py-2">
      <LoaderCircle class="size-(--knovum-icon-size-xs) animate-spin text-accent" :stroke-width="1.7" />
      <span class="min-w-0 flex-1 text-caption font-medium text-foreground-secondary">
        {{ t("spaces.transcript.mediaTranscriptLive") }}
      </span>
      <span class="text-micro text-foreground-tertiary tabular-nums">{{ preview.progress }}%</span>
    </header>
    <div class="max-h-52 overflow-hidden">
      <div
        v-for="segment in recent"
        :key="`${segment.startMs}:${segment.endMs}`"
        class="flex gap-2.5 border-b border-divider/60 px-2.5 py-2 last:border-b-0"
      >
        <span class="w-10 shrink-0 pt-0.5 text-micro text-foreground-tertiary tabular-nums">
          {{ formatDurationMs(segment.startMs) }}
        </span>
        <p class="line-clamp-2 min-w-0 flex-1 text-caption leading-relaxed text-foreground-secondary">
          {{ segment.text }}
        </p>
      </div>
    </div>
    <footer class="border-t border-divider px-2.5 py-1.5 text-micro text-foreground-tertiary">
      {{ t("spaces.transcript.mediaTranscriptLiveHint", { language: preview.language }) }}
    </footer>
  </section>
</template>
