<script setup lang="ts">
import type { GroundedDocumentSource } from "../../knowledge-derivations/catalog/grounded-document";
import type { GroundedDocumentDraft } from "../../knowledge-derivations/grounded-document";
import type { CreationDialogId } from "../../knowledge-derivations/selection/creation-dialog";
import type { VideoDraft } from "../../knowledge-derivations/video/generation";
import type { VisualizationDraft } from "../../knowledge-derivations/visualization";
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
