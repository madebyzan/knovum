<script setup lang="ts">
import { FileText } from "@lucide/vue";
import { camelCase } from "es-toolkit";
import { computed } from "vue";
import { useI18n } from "vue-i18n";

import UiScrollEdgeFade from "../../../../shared/ui/UiScrollEdgeFade.vue";
import { formatByteSize } from "../../assets/format-byte-size";
import SpaceInspectorSection from "../../inspection/presentation/SpaceInspectorSection.vue";
import SpaceNodeCapabilitySections from "../../inspection/presentation/SpaceNodeCapabilitySections.vue";
import SpaceNodeMetadata from "../../inspection/presentation/SpaceNodeMetadata.vue";
import SpaceVisualAnalysis from "../../inspection/presentation/SpaceVisualAnalysis.vue";
import SpaceDerivedInspection from "../../knowledge-derivations/presentation/SpaceDerivedInspection.vue";
import SpaceNodeKnowledgeSections from "../../knowledge/presentation/SpaceNodeKnowledgeSections.vue";
import SpaceRelatedContent from "../../knowledge/presentation/SpaceRelatedContent.vue";
import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import { isSpaceManagedAssetPayload, isSpaceWebsitePayload } from "../../model/nodes/capabilities";

const props = defineProps<{ node: SpaceCanvasNode }>();
const { d, t } = useI18n();
const visual = computed(() => props.node.data.kind === "image" || props.node.data.kind === "video");
const kindLabel = computed(() => t(`spaces.nodeTypes.${camelCase(props.node.data.kind)}`));
const summaryBits = computed(() => {
  const data = props.node.data;
  const parts: string[] = [];
  if (isSpaceWebsitePayload(data) && data.siteName) parts.push(data.siteName);
  if (isSpaceManagedAssetPayload(data)) {
    const size = formatByteSize(data.size);
    if (size) parts.push(size);
    if (data.modifiedAt !== undefined) parts.push(d(data.modifiedAt, "short"));
  }
  return parts;
});
</script>

<template>
  <div :key="node.id" data-reader-ignore-selection class="relative h-full overflow-hidden bg-surface-panel">
    <div class="h-full [scrollbar-gutter:stable] overflow-y-auto overscroll-contain pt-1 pb-12">
      <div class="flex flex-wrap items-center gap-x-2 gap-y-1 px-4 pt-3 pb-2.5 text-caption-sm select-none">
        <span class="rounded-md bg-surface-control px-2 py-0.5 font-medium text-fg-secondary">
          {{ kindLabel }}
        </span>
        <span v-for="bit in summaryBits" :key="bit" class="truncate text-fg-tertiary tabular-nums">
          {{ bit }}
        </span>
      </div>
      <div v-if="visual" class="mx-4 border-t border-stroke-divider/70 py-3">
        <SpaceVisualAnalysis :node="node" />
      </div>
      <SpaceNodeCapabilitySections :node="node" />
      <SpaceDerivedInspection :node="node" />
      <SpaceRelatedContent :node-id="node.id" />
      <SpaceNodeKnowledgeSections :node="node" />
      <SpaceInspectorSection
        :title="t('spaces.inspector.basicInformation')"
        :icon="FileText"
        :default-expanded="false"
      >
        <SpaceNodeMetadata :node="node" />
      </SpaceInspectorSection>
    </div>
    <UiScrollEdgeFade surface="panel" top-class="h-5" bottom-class="h-10" />
  </div>
</template>
