<script setup lang="ts">
import type { Editor } from "@tiptap/vue-3";
import { computed, useTemplateRef } from "vue";

import SpaceCitationPreview from "../../conversation/presentation/SpaceCitationPreview.vue";
import type { useCitationHover } from "../../conversation/presentation/use-citation-hover";
import type { GroundedCitation } from "../../knowledge/model/schema/grounded-citation";
import { useSpaceStore } from "../../session/space-store";
import type { SpaceTiptapHeading } from "../tiptap-outline";
import type { SlashCommandController } from "../tiptap-slash-command";
import { listSpaceAiReferenceCandidates } from "../tiptap-space-references";
import type { SpaceTiptapWritingTools } from "../use-space-tiptap-writing-tools";
import SpaceDocumentOutline from "./SpaceDocumentOutline.vue";
import SpaceTiptapAiComposer from "./SpaceTiptapAiComposer.vue";
import SpaceTiptapSlashMenu from "./SpaceTiptapSlashMenu.vue";

const props = defineProps<{
  editor: Editor | undefined;
  nodeId: string;
  writing: SpaceTiptapWritingTools;
  fullWidth: boolean;
  sourceMode: boolean;
  slashController: SlashCommandController;
  citationHover: ReturnType<typeof useCitationHover>;
  headings: readonly SpaceTiptapHeading[];
  activeHeadingId: string | undefined;
}>();

const emit = defineEmits<{
  openCitation: [citation: GroundedCitation];
  revealHeading: [position: number];
}>();

const store = useSpaceStore();
const outline = useTemplateRef<InstanceType<typeof SpaceDocumentOutline>>("outline");

const aiReferences = computed(() =>
  props.writing.phase.value === "closed" ? [] : listSpaceAiReferenceCandidates(store.nodes, props.nodeId),
);

function handleOpenCitation(citation: GroundedCitation) {
  props.citationHover.dismiss();
  emit("openCitation", citation);
}

defineExpose({ focusOutline: () => void outline.value?.focus() });
</script>

<template>
  <SpaceTiptapAiComposer
    v-if="editor && !sourceMode"
    :editor="editor"
    :writing="writing"
    :references="aiReferences"
    :full-width="fullWidth"
  />
  <SpaceTiptapSlashMenu :controller="slashController" />
  <SpaceCitationPreview
    :state="citationHover.active.value"
    @open="handleOpenCitation"
    @preview-enter="citationHover.onPreviewPointerEnter"
    @preview-leave="citationHover.onPreviewPointerLeave"
  />
  <SpaceDocumentOutline
    v-if="!sourceMode"
    ref="outline"
    :headings="headings"
    :active-id="activeHeadingId"
    @select="(pos) => emit('revealHeading', pos)"
  />
</template>
