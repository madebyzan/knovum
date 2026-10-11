<script setup lang="ts">
import { useI18n } from "vue-i18n";

import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import type { SpaceRelation } from "../../model/relations/relation";

defineProps<{
  relations: readonly SpaceRelation[];
  nodes: readonly SpaceCanvasNode[];
  objectTitles: readonly string[];
}>();
const { t } = useI18n();
</script>

<template>
  <div class="h-full overflow-y-auto overscroll-contain px-4 pt-4 pb-3">
    <h2 class="text-title font-semibold text-fg-primary">
      {{ t("spaces.inspector.itemsSelected", { count: relations.length + objectTitles.length }) }}
    </h2>
    <ul v-if="objectTitles.length" class="mt-3 space-y-2 text-caption text-fg-secondary">
      <li v-for="(title, index) in objectTitles" :key="index">{{ title }}</li>
    </ul>
    <section
      v-for="relation in relations"
      :key="relation.id"
      class="mt-4 border-t border-stroke-divider pt-3"
    >
      <h3 class="text-label font-medium text-fg-primary">
        {{ t(`spaces.knowledge.relations.kinds.${relation.kind}`) }}
      </h3>
      <p class="mt-1 text-caption text-fg-secondary">
        {{ nodes.find(({ id }) => id === relation.sourceId)?.data.title }}
        → {{ nodes.find(({ id }) => id === relation.targetId)?.data.title }}
      </p>
      <template v-if="relation.rationale">
        <h4 class="mt-3 text-caption-sm text-fg-tertiary">
          {{ t("spaces.knowledge.relations.rationale.label") }}
        </h4>
        <p class="mt-1 text-caption whitespace-pre-wrap text-fg-secondary">{{ relation.rationale }}</p>
      </template>
    </section>
  </div>
</template>
