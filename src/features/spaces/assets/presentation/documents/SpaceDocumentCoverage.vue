<script setup lang="ts">
import { computed, shallowRef, watch } from "vue";

import { useRecognizedText } from "../../../enrichment/use-recognized-text";
import type { SpaceCanvasNode } from "../../../model/nodes/canvas-node";
import { useSpaceStore } from "../../../session/space-store";
import { documentResourceExportTarget } from "../../export/document-resource";
import { projectDocumentResources } from "./document-resource-projection";
import SpaceDocumentResourceGrid from "./SpaceDocumentResourceGrid.vue";
import SpaceDocumentResourceViewer from "./SpaceDocumentResourceViewer.vue";

const props = defineProps<{ node: SpaceCanvasNode }>();
const store = useSpaceStore();
const report = computed(() =>
  props.node.data.kind === "file" ? props.node.data.documentCoverage : undefined,
);
const recognizedText = useRecognizedText(
  computed(() => store.activeSpaceId),
  computed(() => props.node.id),
  computed(() =>
    props.node.data.kind === "file" ? props.node.data.textRecognition?.sourceRevision : undefined,
  ),
);
const resources = computed(() => (report.value ? projectDocumentResources(report.value) : []));
const selectedResourceId = shallowRef<string>();
const selectedResource = computed(() =>
  resources.value.find(({ resource }) => resource.id === selectedResourceId.value),
);
const exportTarget = computed(() => {
  const spaceId = store.activeSpaceId;
  const item = selectedResource.value;
  if (!spaceId || !item) return undefined;
  return documentResourceExportTarget({
    spaceId,
    nodeId: props.node.id,
    resourceId: item.resource.id,
    name: `${props.node.data.title} · ${item.locationLabel}`,
  });
});
const viewerOpen = computed({
  get: () => selectedResourceId.value !== undefined,
  set: (open) => {
    if (!open) selectedResourceId.value = undefined;
  },
});
watch(
  () => props.node.id,
  () => {
    selectedResourceId.value = undefined;
  },
);
</script>

<template>
  <SpaceDocumentResourceGrid
    v-if="report && resources.length"
    :key="node.id"
    :resources="resources"
    :source-revision="report.sourceRevision"
    @select="selectedResourceId = $event.resource.id"
  />
  <SpaceDocumentResourceViewer
    v-if="report && selectedResource"
    v-model:open="viewerOpen"
    :item="selectedResource"
    :source-revision="report.sourceRevision"
    :export-target="exportTarget"
    :recognized-text="recognizedText(selectedResource.resource.id)"
  />
</template>
