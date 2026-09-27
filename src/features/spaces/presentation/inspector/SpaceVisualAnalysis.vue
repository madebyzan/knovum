<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";

import UiBadge from "../../../../shared/ui/UiBadge.vue";
import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";

const props = defineProps<{ node: SpaceCanvasNode }>();
const { t } = useI18n();
const knowledge = computed(() => {
  const data = props.node.data;
  return data.kind === "image" || data.kind === "video" ? data.visualKnowledge : undefined;
});
</script>

<template>
  <div>
    <template v-if="knowledge">
      <p class="text-body leading-relaxed text-fg-secondary select-text">
        {{ knowledge.description }}
      </p>
      <div v-if="knowledge.detectedText" class="mt-3">
        <h4 class="text-caption font-medium text-fg-tertiary">
          {{ t("spaces.inspector.visualDetectedText") }}
        </h4>
        <pre
          class="mt-1 font-sans text-caption leading-relaxed whitespace-pre-wrap text-fg-secondary select-text"
          >{{ knowledge.detectedText }}</pre>
      </div>
      <div v-if="knowledge.tags.length" class="mt-3 flex flex-wrap gap-1">
        <UiBadge v-for="tag in knowledge.tags" :key="tag" size="sm">
          {{ tag }}
        </UiBadge>
      </div>
    </template>
    <p v-else class="text-caption leading-relaxed text-fg-tertiary">
      {{ t("spaces.knowledge.visual.pending") }}
    </p>
  </div>
</template>
