<script setup lang="ts">
import type { GroundedDocumentSource } from "../catalog/grounded-document";
import type { GroundedDocumentDraft } from "../grounded-document";
import type { CreationDialogId } from "../selection/creation-dialog";
import type { VideoDraft } from "../video/generation";
import type { VisualizationDraft } from "../visualization";
import GroundedDocumentDialog from "./GroundedDocumentDialog.vue";
import VideoGenerationDialog from "./VideoGenerationDialog.vue";
import VisualizationDialog from "./VisualizationDialog.vue";

const open = defineModel<CreationDialogId | undefined>("open");
defineProps<{
  sourceTitle: string;
  sources: GroundedDocumentSource[];
}>();
const emit = defineEmits<{
  document: [draft: GroundedDocumentDraft];
  visualization: [draft: VisualizationDraft];
  video: [draft: VideoDraft];
}>();
</script>

<template>
  <GroundedDocumentDialog
    :open="open === 'document'"
    :sources="sources"
    @update:open="!$event && (open = undefined)"
    @submit="emit('document', $event)"
  />
  <VisualizationDialog
    :open="open === 'visualization'"
    :sources="sources"
    @update:open="!$event && (open = undefined)"
    @submit="emit('visualization', $event)"
  />
  <VideoGenerationDialog
    :open="open === 'video'"
    :source-title="sourceTitle"
    @update:open="!$event && (open = undefined)"
    @submit="emit('video', $event)"
  />
</template>
