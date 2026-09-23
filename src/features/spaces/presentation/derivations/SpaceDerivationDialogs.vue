<script setup lang="ts">
import type { GroundedDocumentSource } from "../../knowledge-derivations/catalog/grounded-document";
import type { GroundedDocumentDraft } from "../../knowledge-derivations/grounded-document";
import type { CreationDialogId } from "../../knowledge-derivations/selection/creation-dialog";
import type { VideoDraft } from "../../knowledge-derivations/video/generation";
import type { VisualizationDraft } from "../../knowledge-derivations/visualization";
import type { MeetingContentPackageDraft } from "../../knowledge-derivations/workflows/meeting-content-package";
import type { PresentationGenerationDraft } from "../../knowledge-derivations/workflows/presentation-production";
import type { SocialContentPackageDraft } from "../../knowledge-derivations/workflows/social-content-package";
import GroundedDocumentDialog from "./GroundedDocumentDialog.vue";
import MeetingContentPackageDialog from "./MeetingContentPackageDialog.vue";
import PresentationGenerationDialog from "./PresentationGenerationDialog.vue";
import SocialContentPackageDialog from "./SocialContentPackageDialog.vue";
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
  social: [draft: SocialContentPackageDraft];
  meeting: [draft: MeetingContentPackageDraft];
  presentation: [draft: PresentationGenerationDraft];
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
  <SocialContentPackageDialog
    :open="open === 'social-content-package'"
    :sources="sources"
    @update:open="!$event && (open = undefined)"
    @submit="emit('social', $event)"
  />
  <MeetingContentPackageDialog
    :open="open === 'meeting-content-package'"
    :source-title="sourceTitle"
    @update:open="!$event && (open = undefined)"
    @submit="emit('meeting', $event)"
  />
  <PresentationGenerationDialog
    :open="open === 'presentation'"
    :source-title="sourceTitle"
    @update:open="!$event && (open = undefined)"
    @submit="emit('presentation', $event)"
  />
</template>
