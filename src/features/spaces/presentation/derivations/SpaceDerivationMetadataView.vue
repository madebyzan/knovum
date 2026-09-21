<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";

import type { VisualizationMetadata } from "../../knowledge-derivations/image/metadata";
import type { GeneratedVideoMetadata } from "../../knowledge-derivations/video/metadata";
import SpaceDerivationSources from "./SpaceDerivationSources.vue";

const props = defineProps<{ metadata: VisualizationMetadata | GeneratedVideoMetadata }>();
const { t } = useI18n();

const isImage = computed(() => "intent" in props.metadata);
const description = computed(() =>
  isImage.value
    ? (props.metadata as VisualizationMetadata).intent
    : (props.metadata as GeneratedVideoMetadata).prompt,
);
const metaLine = computed(() => {
  const date = new Date(props.metadata.createdAt).toLocaleString();
  if (isImage.value) {
    const meta = props.metadata as VisualizationMetadata;
    return `${meta.style} · ${meta.aspectRatio} · ${date}`;
  }
  const meta = props.metadata as GeneratedVideoMetadata;
  return `${meta.aspectRatio} · ${meta.durationSeconds}s · ${date}`;
});
</script>

<template>
  <div class="space-y-4 text-caption">
    <div class="space-y-1 text-foreground-secondary">
      <p>
        {{ t("spaces.creation.generatedWith", { provider: metadata.provider, model: metadata.model }) }}
      </p>
      <p>{{ description }}</p>
      <p class="text-foreground-tertiary">{{ metaLine }}</p>
    </div>
    <SpaceDerivationSources :sources="metadata.sources" />
  </div>
</template>
