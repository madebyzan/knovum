<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";

import AiExecutionSourceLabel from "../../../../foundation/ai/AiExecutionSourceLabel.vue";
import UiExpandableText from "../../../../shared/ui/UiExpandableText.vue";
import type { GeneratedVideoMetadata } from "../../contracts/generated/GeneratedVideoMetadata";
import type { VisualizationMetadata } from "../../contracts/generated/VisualizationMetadata";
import SpaceDerivationSources from "./SpaceDerivationSources.vue";

const props = defineProps<{ metadata: VisualizationMetadata | GeneratedVideoMetadata }>();
const { d } = useI18n();

const description = computed(() =>
  "intent" in props.metadata ? props.metadata.intent : props.metadata.prompt,
);
const metaLine = computed(() => {
  const date = d(props.metadata.createdAt, "short");
  if ("intent" in props.metadata) {
    const meta = props.metadata;
    return `${meta.aspectRatio} · ${date}`;
  }
  const meta = props.metadata;
  return `${meta.aspectRatio} · ${meta.durationSeconds}s · ${date}`;
});
</script>

<template>
  <div class="space-y-3 text-caption">
    <div class="space-y-1.5">
      <div class="flex flex-wrap items-baseline justify-between gap-x-2 gap-y-0.5 text-caption-sm">
        <span class="font-medium text-fg-secondary">
          <AiExecutionSourceLabel :source="metadata.executionSource" />
        </span>
        <span class="text-fg-tertiary tabular-nums">{{ metaLine }}</span>
      </div>
      <UiExpandableText
        v-if="description"
        :text="description"
        :clamp-lines="3"
        class="text-caption leading-relaxed text-fg-secondary"
      />
    </div>
    <SpaceDerivationSources :sources="metadata.sources" />
  </div>
</template>
