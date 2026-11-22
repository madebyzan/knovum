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
import { formatDurationMs } from "../../media/format-duration";
import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import {
  isSpaceManagedAssetPayload,
  isSpaceMediaPayload,
  isSpaceWebsitePayload,
} from "../../model/nodes/capabilities";

const props = defineProps<{ node: SpaceCanvasNode }>();
const { d, t } = useI18n();
const visual = computed(() => props.node.data.kind === "image" || props.node.data.kind === "video");
const kindLabel = computed(() => t(`spaces.nodeTypes.${camelCase(props.node.data.kind)}`));
const summaryBits = computed(() => {
  const data = props.node.data;
  return [
    isSpaceWebsitePayload(data) ? data.siteName : undefined,
    isSpaceMediaPayload(data) && data.durationSeconds
      ? formatDurationMs(data.durationSeconds * 1000)
      : undefined,
    visualDimensions(data),
    ...managedAssetBits(data),
  ].filter((bit): bit is string => Boolean(bit));
});

function visualDimensions(data: SpaceCanvasNode["data"]): string | undefined {
  if (data.kind !== "image" && data.kind !== "video") return undefined;
  return data.intrinsicWidth && data.intrinsicHeight
    ? `${data.intrinsicWidth}×${data.intrinsicHeight}`
    : undefined;
}

function managedAssetBits(data: SpaceCanvasNode["data"]): (string | undefined)[] {
  if (!isSpaceManagedAssetPayload(data)) return [];
  return [formatByteSize(data.size), data.modifiedAt !== undefined ? d(data.modifiedAt, "short") : undefined];
}
</script>

<template>
  <div :key="node.id" data-reader-ignore-selection class="relative h-full overflow-hidden bg-surface-panel">
    <div class="h-full [scrollbar-gutter:stable] overflow-y-auto overscroll-contain pt-1 pb-12">
      <div
        class="flex flex-wrap items-center gap-x-1.5 gap-y-0.5 px-4 pt-3 pb-2.5 text-caption text-fg-tertiary select-none"
      >
        <span class="font-medium text-fg-secondary">{{ kindLabel }}</span>
        <template v-for="bit in summaryBits" :key="bit">
          <span aria-hidden="true" class="text-fg-tertiary/55">·</span>
          <span class="truncate tabular-nums">{{ bit }}</span>
        </template>
      </div>
      <SpaceVisualAnalysis v-if="visual" :node="node" />
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
