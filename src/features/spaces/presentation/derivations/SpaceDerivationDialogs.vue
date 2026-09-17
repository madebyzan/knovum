<script setup lang="ts">
import type { GroundedDocumentSource } from "../../knowledge-derivations/catalog/grounded-document";
import type { GroundedDocumentDraft } from "../../knowledge-derivations/grounded-document";
import type { CreationDialogId } from "../../knowledge-derivations/selection/creation-dialog";
import type { KnowledgeVideoDraft } from "../../knowledge-derivations/video/knowledge-video";
import type { VisualizationDraft } from "../../knowledge-derivations/visualization";
import type { KnowledgePresentationDraft } from "../../knowledge-derivations/workflows/knowledge-presentation";
import type { MeetingVideoPackageDraft } from "../../knowledge-derivations/workflows/meeting-video-package";
import type { SocialContentKitDraft } from "../../knowledge-derivations/workflows/social-content-kit";
import GroundedDocumentDialog from "./GroundedDocumentDialog.vue";
import KnowledgePresentationDialog from "./KnowledgePresentationDialog.vue";
import KnowledgeVideoDialog from "./KnowledgeVideoDialog.vue";
import KnowledgeVisualizationDialog from "./KnowledgeVisualizationDialog.vue";
import MeetingVideoPackageDialog from "./MeetingVideoPackageDialog.vue";
import SocialContentKitDialog from "./SocialContentKitDialog.vue";

const open = defineModel<CreationDialogId | undefined>("open");
defineProps<{
  sourceTitle: string;
  sources: GroundedDocumentSource[];
}>();
const emit = defineEmits<{
  document: [draft: GroundedDocumentDraft];
  visualization: [draft: VisualizationDraft];
  video: [draft: KnowledgeVideoDraft];
  social: [draft: SocialContentKitDraft];
  meeting: [draft: MeetingVideoPackageDraft];
  presentation: [draft: KnowledgePresentationDraft];
}>();
</script>

<template>
  <GroundedDocumentDialog
    :open="open === 'document'"
    :sources="sources"
    @update:open="!$event && (open = undefined)"
    @submit="emit('document', $event)"
  />
  <KnowledgeVisualizationDialog
    :open="open === 'visualization'"
    :sources="sources"
    @update:open="!$event && (open = undefined)"
    @submit="emit('visualization', $event)"
  />
  <KnowledgeVideoDialog
    :open="open === 'video'"
    :source-title="sourceTitle"
    @update:open="!$event && (open = undefined)"
    @submit="emit('video', $event)"
  />
  <SocialContentKitDialog
    :open="open === 'social-content-kit'"
    :sources="sources"
    @update:open="!$event && (open = undefined)"
    @submit="emit('social', $event)"
  />
  <MeetingVideoPackageDialog
    :open="open === 'meeting-video-package'"
    :source-title="sourceTitle"
    @update:open="!$event && (open = undefined)"
    @submit="emit('meeting', $event)"
  />
  <KnowledgePresentationDialog
    :open="open === 'presentation'"
    :source-title="sourceTitle"
    @update:open="!$event && (open = undefined)"
    @submit="emit('presentation', $event)"
  />
</template>
