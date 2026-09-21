<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";

import { classifyFilePresentation, usesDocumentPreview } from "../../../assets/file-presentation";
import type { SpaceNodePayload } from "../../../model/nodes/node-payload";
import SpaceDocumentPreview from "../../inspector/SpaceDocumentPreview.vue";
import { spaceNodeSummary } from "./space-canvas-node-content";
import { spaceCanvasNodePresentation } from "./space-canvas-node-presentation";
import SpaceCanvasArchiveEntries from "./SpaceCanvasArchiveEntries.vue";
import SpaceCanvasCode from "./SpaceCanvasCode.vue";
import SpaceCanvasMarkdown from "./SpaceCanvasMarkdown.vue";
import SpaceCanvasMedia from "./SpaceCanvasMedia.vue";
import SpaceCanvasProcessingState from "./SpaceCanvasProcessingState.vue";

const props = defineProps<{
  id: string;
  data: SpaceNodePayload;
  detailLevel: "compact" | "summary" | "full";
  interactive: boolean;
}>();
const { t } = useI18n();
const presentation = computed(() => spaceCanvasNodePresentation(props.data));
const entries = computed(() => (props.data.kind === "file" && props.data.entries ? props.data.entries : []));
const documentPreview = computed(() => {
  if (props.data.kind !== "file") return undefined;
  return usesDocumentPreview(classifyFilePresentation(props.data.fileName ?? props.data.title))
    ? props.data
    : undefined;
});
const documentPreviewProps = computed(() => {
  const data = documentPreview.value;
  if (!data) return undefined;
  return {
    title: data.title,
    ...(data.previewPath === undefined ? {} : { previewPath: data.previewPath }),
    ...(data.previewText === undefined ? {} : { previewText: data.previewText }),
  };
});
const text = computed(() => spaceNodeSummary(props.data) ?? presentation.value.subtitle);
const processing = computed(() =>
  "processingState" in props.data &&
  (props.data.processingState === "importing" || props.data.processingState === "processing")
    ? props.data.processingState
    : undefined,
);
const failed = computed(() => "processingState" in props.data && props.data.processingState === "failed");
</script>

<template>
  <SpaceCanvasMedia
    v-if="data.kind === 'audio' || (data.kind === 'video' && detailLevel === 'full')"
    :id="id"
    :data="data"
  />
  <SpaceCanvasArchiveEntries
    v-else-if="entries.length > 0"
    :entries="entries"
    :detail-level="detailLevel"
    :interactive="interactive"
  />
  <SpaceDocumentPreview
    v-else-if="
      documentPreviewProps &&
      detailLevel === 'full' &&
      (documentPreviewProps.previewPath || documentPreviewProps.previewText)
    "
    v-bind="documentPreviewProps"
    fill
  />
  <img
    v-else-if="presentation.previewSource"
    :src="presentation.previewSource"
    :alt="presentation.title"
    class="block min-h-0 flex-1 object-contain"
    draggable="false"
  />
  <SpaceCanvasMarkdown
    v-else-if="data.kind === 'markdown' && detailLevel !== 'compact'"
    :data="data"
    :detail-level="detailLevel"
  />
  <SpaceCanvasCode
    v-else-if="data.kind === 'code' && detailLevel !== 'compact'"
    :data="data"
    :detail-level="detailLevel"
    :interactive="interactive"
  />
  <SpaceCanvasProcessingState
    v-else-if="processing"
    :label="
      processing === 'importing' ? t('spaces.transfer.importingItem') : t('spaces.transfer.importProcessing')
    "
  />
  <SpaceCanvasProcessingState v-else-if="failed" :label="t('spaces.processing.processingFailed')" failed />
  <div v-else class="min-h-0 flex-1 overflow-hidden px-3 py-2.5">
    <p
      class="leading-relaxed whitespace-pre-wrap text-foreground-secondary"
      :class="[
        presentation.mono && 'font-mono',
        detailLevel === 'compact' ? 'line-clamp-2 text-caption' : 'line-clamp-8 text-body',
      ]"
    >
      {{ text }}
    </p>
  </div>
</template>
