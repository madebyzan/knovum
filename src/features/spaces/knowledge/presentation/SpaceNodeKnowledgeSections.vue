<script setup lang="ts">
import { Layers, Tag } from "@lucide/vue";
import { computed } from "vue";

import SpaceInspectorSection from "../../inspection/presentation/SpaceInspectorSection.vue";
import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import { useSpaceStore } from "../../session/space-store";
import SpaceKnowledgeHistory from "./SpaceKnowledgeHistory.vue";
import SpaceRelations from "./SpaceRelations.vue";
import SpaceTags from "./SpaceTags.vue";

const props = defineProps<{ node: SpaceCanvasNode }>();
const store = useSpaceStore();
const hasRelations = computed(() =>
  store.relations.some(
    (relation) => relation.sourceId === props.node.id || relation.targetId === props.node.id,
  ),
);
</script>

<template>
  <SpaceInspectorSection
    :title="$t('spaces.knowledge.relations.title')"
    :icon="Layers"
    :default-expanded="hasRelations"
  >
    <SpaceRelations />
  </SpaceInspectorSection>

  <SpaceKnowledgeHistory v-if="node.data.kind === 'object'" :node="node" />

  <SpaceInspectorSection :title="$t('spaces.tags.title')" :icon="Tag" :default-expanded="true">
    <SpaceTags mode="selection" />
  </SpaceInspectorSection>
</template>
