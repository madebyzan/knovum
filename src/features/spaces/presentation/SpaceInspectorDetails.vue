<script setup lang="ts">
import { FileText, Tag } from "@lucide/vue";
import { computed } from "vue";

import { useCanvasSelectionProjection } from "../canvas/editor-session";
import type { SpaceCanvasNode } from "../model/nodes/canvas-node";
import { useSpaceStore } from "../session/space-store";
import SpaceKnowledgeHealth from "./knowledge/SpaceKnowledgeHealth.vue";
import SpaceDerivedInspection from "./SpaceDerivedInspection.vue";
import SpaceInspectorSection from "./SpaceInspectorSection.vue";
import SpaceNodeIdentity from "./SpaceNodeIdentity.vue";
import SpaceNodeKnowledgeSections from "./SpaceNodeKnowledgeSections.vue";
import SpaceNodeMetadata from "./SpaceNodeMetadata.vue";
import SpaceNodeTypeDetails from "./SpaceNodeTypeDetails.vue";
import SpaceTags from "./SpaceTags.vue";

const store = useSpaceStore();
const { selectedNodes } = useCanvasSelectionProjection(() => store.nodes);
const selectedNode = computed(() => (selectedNodes.value.length === 1 ? selectedNodes.value[0] : undefined));

function saveTitle(node: SpaceCanvasNode, value: string) {
  return store.saveNodeData(node.id, node.data, { ...node.data, title: value });
}
</script>

<template>
  <div v-if="selectedNode" class="h-full overflow-y-auto">
    <SpaceNodeIdentity :node="selectedNode" :save-title="saveTitle" />

    <SpaceNodeTypeDetails :node="selectedNode" />
    <SpaceDerivedInspection :node="selectedNode" />
    <SpaceNodeKnowledgeSections :node="selectedNode" />
    <SpaceInspectorSection
      :title="$t('spaces.inspector.basicInformation')"
      :icon="FileText"
      :default-expanded="false"
    >
      <SpaceNodeMetadata :node="selectedNode" />
    </SpaceInspectorSection>
  </div>
  <div v-else-if="selectedNodes.length > 1" class="h-full overflow-y-auto">
    <SpaceInspectorSection :title="$t('spaces.navigation.tags')" :icon="Tag" :default-expanded="true">
      <SpaceTags mode="selection" />
    </SpaceInspectorSection>
  </div>
  <div v-else class="h-full overflow-y-auto px-3 py-3">
    <SpaceKnowledgeHealth />
  </div>
</template>
