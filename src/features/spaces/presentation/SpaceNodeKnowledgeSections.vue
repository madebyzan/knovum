<script setup lang="ts">
import { Layers, Tag } from "@lucide/vue";

import type { SpaceCanvasNode } from "../model/nodes/canvas-node";
import SpaceInspectorSection from "./inspector/SpaceInspectorSection.vue";
import SpaceKnowledgeHistory from "./knowledge/SpaceKnowledgeHistory.vue";
import SpaceRelations from "./knowledge/SpaceRelations.vue";
import SpaceTags from "./SpaceTags.vue";

defineProps<{ node: SpaceCanvasNode }>();
</script>

<template>
  <SpaceInspectorSection
    :title="$t('spaces.knowledge.relations.title')"
    :icon="Layers"
    :default-expanded="true"
  >
    <SpaceRelations />
  </SpaceInspectorSection>

  <SpaceKnowledgeHistory v-if="node.data.kind === 'object'" :node="node" />

  <SpaceInspectorSection :title="$t('spaces.tags.title')" :icon="Tag" :default-expanded="true">
    <SpaceTags mode="selection" />
  </SpaceInspectorSection>
</template>
