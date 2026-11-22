<script setup lang="ts">
import { Layers, Tag } from "@lucide/vue";
import { computed } from "vue";

import SpaceInspectorSection from "../../inspection/presentation/SpaceInspectorSection.vue";
import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import { useSpaceStore } from "../../session/space-store";
import SpaceKnowledgeHistory from "./SpaceKnowledgeHistory.vue";
import SpaceProvenanceSources from "./SpaceProvenanceSources.vue";
import SpaceRelations from "./SpaceRelations.vue";
import SpaceSourceUpdates from "./SpaceSourceUpdates.vue";
import SpaceTags from "./SpaceTags.vue";

const props = defineProps<{ node: SpaceCanvasNode }>();
const store = useSpaceStore();
const relationCount = computed(
  () =>
    store.relations.filter(
      (relation) => relation.sourceId === props.node.id || relation.targetId === props.node.id,
    ).length,
);
const tagCount = computed(() => props.node.data.tags?.length ?? 0);
</script>

<template>
  <SpaceSourceUpdates :key="node.id" :node-id="node.id" />

  <SpaceProvenanceSources
    v-if="node.data.kind !== 'object'"
    class="mx-4 border-t border-stroke-divider/70 py-3"
    target-kind="node"
    :target-id="node.id"
    removable
  />

  <SpaceInspectorSection
    :title="$t('spaces.knowledge.relations.title')"
    :icon="Layers"
    :count="relationCount"
    :default-expanded="relationCount > 0"
  >
    <SpaceRelations :node="node" />
  </SpaceInspectorSection>

  <SpaceKnowledgeHistory v-if="node.data.kind === 'object'" :node="node" />

  <SpaceInspectorSection
    :title="$t('spaces.tags.title')"
    :icon="Tag"
    :count="tagCount"
    :default-expanded="true"
  >
    <SpaceTags :target-nodes="[node]" />
  </SpaceInspectorSection>
</template>
