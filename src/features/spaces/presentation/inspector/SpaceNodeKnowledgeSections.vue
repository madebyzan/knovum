<script setup lang="ts">
import { Layers, Tag } from "@lucide/vue";

import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import SpaceKnowledgeHistory from "../knowledge/SpaceKnowledgeHistory.vue";
import SpaceRelations from "../knowledge/SpaceRelations.vue";
import SpaceInspectorSection from "./SpaceInspectorSection.vue";
import SpaceTags from "./SpaceTags.vue";

defineProps<{ node: SpaceCanvasNode }>();
</script>

<template>
  <SpaceInspectorSection :title="$t('spaces.knowledge.relations')" :icon="Layers" :default-expanded="true">
    <SpaceRelations />
  </SpaceInspectorSection>

  <SpaceKnowledgeHistory v-if="node.data.kind === 'object'" :node="node" />

  <SpaceInspectorSection :title="$t('spaces.navigation.tags')" :icon="Tag" :default-expanded="true">
    <SpaceTags mode="selection" />
  </SpaceInspectorSection>
</template>
