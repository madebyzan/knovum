<script setup lang="ts">
import { FileText, Group, Tag } from "@lucide/vue";
import { computed } from "vue";

import UiButton from "../../../../shared/ui/UiButton.vue";
import { useCanvasSelectionProjection } from "../../canvas/editor-session";
import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import { useSpaceStore } from "../../session/space-store";
import { useCanvasSelectionActions } from "../canvas/use-canvas-selection-actions";
import SpaceDerivedInspection from "../derivations/SpaceDerivedInspection.vue";
import SpaceKnowledgeHealth from "../knowledge/SpaceKnowledgeHealth.vue";
import SpaceNodeIdentity from "../SpaceNodeIdentity.vue";
import SpaceNodeKnowledgeSections from "../SpaceNodeKnowledgeSections.vue";
import SpaceRelatedContent from "../SpaceRelatedContent.vue";
import SpaceTags from "../SpaceTags.vue";
import SpaceInspectorSection from "./SpaceInspectorSection.vue";
import SpaceNodeCapabilitySections from "./SpaceNodeCapabilitySections.vue";
import SpaceNodeMetadata from "./SpaceNodeMetadata.vue";
import SpaceNodeOverview from "./SpaceNodeOverview.vue";

const store = useSpaceStore();
const { selectedNodes } = useCanvasSelectionProjection(() => store.nodes);
const selectionActions = useCanvasSelectionActions();
const selectedNode = computed(() => (selectedNodes.value.length === 1 ? selectedNodes.value[0] : undefined));

function saveTitle(node: SpaceCanvasNode, value: string) {
  return store.saveNodeContent(node.id, node.data, { ...node.data, title: value });
}
</script>

<template>
  <div v-if="selectedNode" :key="selectedNode.id" class="h-full overflow-y-auto overscroll-contain">
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
  <div v-else-if="selectedNodes.length > 1" class="h-full overflow-y-auto overscroll-contain pt-4">
    <div class="px-4 pb-4">
      <h2 class="text-title font-semibold text-foreground">
        {{ $t("spaces.inspector.itemsSelected", { count: selectedNodes.length }) }}
      </h2>
      <ul class="mt-3 divide-y divide-divider rounded-section-inner border border-divider">
        <li
          v-for="node in selectedNodes"
          :key="node.id"
          class="truncate px-3 py-2 text-caption text-foreground-secondary"
        >
          {{ node.data.title }}
        </li>
      </ul>
      <UiButton
        v-if="selectionActions.state.value.canCreateGroup"
        size="sm"
        class="mt-3"
        @click="selectionActions.createGroup"
      >
        <Group class="size-(--knovum-icon-size-xs)" />
        {{ $t("spaces.canvas.groups.create") }}
      </UiButton>
    </div>
    <SpaceInspectorSection :title="$t('spaces.tags.title')" :icon="Tag" :default-expanded="true">
      <SpaceTags mode="selection" />
    </SpaceInspectorSection>
  </div>
  <div v-else class="h-full overflow-y-auto px-3 py-3">
    <SpaceKnowledgeHealth />
  </div>
</template>
