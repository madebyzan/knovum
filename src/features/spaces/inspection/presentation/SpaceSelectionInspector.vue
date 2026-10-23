<script setup lang="ts">
import { FileText, Tag } from "@lucide/vue";
import { computed } from "vue";

import { getCanvasEditorSession } from "../../canvas/editor-session";
import { useCanvasSelectionProjection } from "../../canvas/use-canvas-selection-projection";
import SpaceDerivedInspection from "../../knowledge-derivations/presentation/SpaceDerivedInspection.vue";
import SpaceKnowledgeHealth from "../../knowledge/presentation/SpaceKnowledgeHealth.vue";
import SpaceNodeKnowledgeSections from "../../knowledge/presentation/SpaceNodeKnowledgeSections.vue";
import SpaceRelatedContent from "../../knowledge/presentation/SpaceRelatedContent.vue";
import SpaceRetrievalIndex from "../../knowledge/presentation/SpaceRetrievalIndex.vue";
import SpaceTags from "../../knowledge/presentation/SpaceTags.vue";
import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import { useSpaceStore } from "../../session/space-store";
import SpaceInspectorSection from "./SpaceInspectorSection.vue";
import SpaceNodeCapabilitySections from "./SpaceNodeCapabilitySections.vue";
import SpaceNodeIdentity from "./SpaceNodeIdentity.vue";
import SpaceNodeMetadata from "./SpaceNodeMetadata.vue";
import SpaceNodeOverview from "./SpaceNodeOverview.vue";
import SpaceRelationInspector from "./SpaceRelationInspector.vue";
import SpaceStructureInspector from "./SpaceStructureInspector.vue";

const store = useSpaceStore();
const editor = getCanvasEditorSession();
const { selectedNodes } = useCanvasSelectionProjection(() => store.nodes);
const selectedNode = computed(() => (selectedNodes.value.length === 1 ? selectedNodes.value[0] : undefined));

const selectedRelations = computed(() =>
  store.relations.filter(({ id }) => editor.selection.value.relationIds.includes(id)),
);
const objectTitles = computed(() => [
  ...selectedNodes.value.map(({ data }) => data.title),
  ...store.editSnapshot.structure.flatMap((node) =>
    node.kind !== "content" && editor.selection.value.structureNodeIds.includes(node.id) ? [node.title] : [],
  ),
]);

function saveTitle(node: SpaceCanvasNode, value: string) {
  return store.saveNodeContent({
    id: node.id,
    expectedContent: node.data,
    data: { ...node.data, title: value },
  });
}
</script>

<template>
  <SpaceRelationInspector
    v-if="selectedRelations.length"
    :relations="selectedRelations"
    :nodes="store.nodes"
    :object-titles="objectTitles"
  />
  <SpaceStructureInspector v-else-if="editor.selection.value.structureNodeIds.length" />
  <div
    v-else-if="selectedNode"
    :key="selectedNode.id"
    class="h-full [scrollbar-gutter:stable] overflow-y-auto overscroll-contain pb-3"
  >
    <SpaceNodeIdentity :node="selectedNode" :save-title="saveTitle" />
    <SpaceNodeOverview :node="selectedNode" />
    <SpaceNodeCapabilitySections :node="selectedNode" />
    <SpaceDerivedInspection :node="selectedNode" />
    <SpaceRelatedContent :node-id="selectedNode.id" />
    <SpaceNodeKnowledgeSections :node="selectedNode" />
    <SpaceInspectorSection
      :title="$t('spaces.inspector.basicInformation')"
      :icon="FileText"
      :default-expanded="false"
    >
      <SpaceNodeMetadata :node="selectedNode" />
    </SpaceInspectorSection>
  </div>
  <div
    v-else-if="selectedNodes.length > 1"
    class="h-full [scrollbar-gutter:stable] overflow-y-auto overscroll-contain pt-4 pb-3"
  >
    <div class="px-4 pb-4">
      <h2 class="text-title font-semibold text-fg-primary">
        {{ $t("spaces.inspector.itemsSelected", { count: selectedNodes.length }) }}
      </h2>
      <ul class="mt-3 divide-y divide-stroke-divider rounded-section-inner border border-stroke-divider">
        <li
          v-for="node in selectedNodes"
          :key="node.id"
          class="truncate px-3 py-2 text-caption text-fg-secondary"
        >
          {{ node.data.title }}
        </li>
      </ul>
    </div>
    <SpaceInspectorSection :title="$t('spaces.tags.title')" :icon="Tag" :default-expanded="true">
      <SpaceTags mode="selection" />
    </SpaceInspectorSection>
  </div>
  <div v-else class="h-full space-y-4 overflow-y-auto px-3 py-3">
    <SpaceRetrievalIndex />
    <SpaceKnowledgeHealth />
  </div>
</template>
