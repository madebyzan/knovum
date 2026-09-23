<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";

import { classifyFilePresentation, usesDocumentPreview } from "../../../assets/file-presentation";
import type { CanvasNodePresentationContext } from "../../../canvas/contract/canvas-detail-profile";
import type { SpaceNodePayload } from "../../../model/nodes/node-payload";
import SpaceDocumentPreview from "../../SpaceDocumentPreview.vue";
import { spaceNodePreviewText } from "./space-canvas-node-content";
import { spaceCanvasNodePresentation } from "./space-canvas-node-presentation";
import SpaceCanvasArchiveEntries from "./SpaceCanvasArchiveEntries.vue";
import SpaceCanvasCode from "./SpaceCanvasCode.vue";
import SpaceCanvasImage from "./SpaceCanvasImage.vue";
import SpaceCanvasMarkdown from "./SpaceCanvasMarkdown.vue";
import SpaceCanvasMedia from "./SpaceCanvasMedia.vue";
import SpaceCanvasProcessingState from "./SpaceCanvasProcessingState.vue";

const props = defineProps<{
  id: string;
  data: SpaceNodePayload;
  presentationContext: CanvasNodePresentationContext;
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
const text = computed(
  () => spaceNodePreviewText(props.data, props.presentationContext) ?? presentation.value.subtitle,
);
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
    v-if="data.kind === 'audio' || (data.kind === 'video' && presentationContext.mode === 'interactive')"
    :id="id"
    :data="data"
  />
  <SpaceCanvasImage v-else-if="data.kind === 'image'" :id="id" :data="data" :interactive="interactive" />
  <SpaceCanvasArchiveEntries
    v-else-if="entries.length > 0"
    :entries="entries"
    :presentation-context="presentationContext"
  />
  <SpaceDocumentPreview
    v-else-if="
      documentPreviewProps &&
      presentationContext.viewport.level === 'full' &&
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
    v-else-if="data.kind === 'markdown'"
    :data="data"
    :presentation-context="presentationContext"
  />
  <SpaceCanvasCode
    v-else-if="data.kind === 'code'"
    :data="data"
    :presentation-context="presentationContext"
    :interactive="interactive"
  />
  <SpaceCanvasProcessingState
    v-else-if="processing"
    :label="
      processing === 'importing'
        ? t('spaces.transfer.importingResource')
        : t('spaces.transfer.importProcessing')
    "
  />
  <SpaceCanvasProcessingState v-else-if="failed" :label="t('spaces.processing.processingFailed')" failed />
  <div v-else class="min-h-0 flex-1 overflow-hidden px-3 py-2.5">
    <p
      class="leading-relaxed whitespace-pre-wrap text-foreground-secondary"
      :class="[
        presentation.mono && 'font-mono',
        presentationContext.viewport.level === 'compact' ? 'text-caption' : 'text-body',
      ]"
    >
      {{ text }}
    </p>
  </div>
</template>
