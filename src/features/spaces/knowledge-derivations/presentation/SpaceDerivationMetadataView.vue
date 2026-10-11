<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";

import AiExecutionSourceLabel from "../../../../foundation/ai/AiExecutionSourceLabel.vue";
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
    return `${meta.style} · ${meta.aspectRatio} · ${date}`;
  }
  const meta = props.metadata;
  return `${meta.aspectRatio} · ${meta.durationSeconds}s · ${date}`;
});
</script>

<template>
  <div class="space-y-4 text-caption">
    <div class="space-y-1 text-fg-secondary">
      <p>
        <AiExecutionSourceLabel :source="metadata.executionSource" />
      </p>
      <p>{{ description }}</p>
      <p class="text-fg-tertiary">{{ metaLine }}</p>
    </div>
    <SpaceDerivationSources :sources="metadata.sources" />
  </div>
</template>
