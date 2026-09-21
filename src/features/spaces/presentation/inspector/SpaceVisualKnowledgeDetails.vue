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
      <p
        class="rounded-control bg-accent-subtle/50 px-2.5 py-2 text-caption leading-relaxed text-foreground select-text"
      >
        {{ knowledge.description }}
      </p>
      <div v-if="knowledge.detectedText" class="mt-3">
        <h4 class="text-micro font-semibold tracking-wide text-foreground-tertiary uppercase">
          {{ t("spaces.inspector.visualDetectedText") }}
        </h4>
        <pre
          class="mt-1 font-sans text-caption leading-relaxed whitespace-pre-wrap text-foreground-secondary select-text"
          >{{ knowledge.detectedText }}</pre>
      </div>
      <div v-if="knowledge.tags.length" class="mt-3 flex flex-wrap gap-1">
        <UiBadge v-for="tag in knowledge.tags" :key="tag" size="sm">
          {{ tag }}
        </UiBadge>
      </div>
    </template>
    <p v-else class="text-caption leading-relaxed text-foreground-tertiary">
      {{ t("spaces.knowledge.visualKnowledgePending") }}
    </p>
  </div>
</template>
