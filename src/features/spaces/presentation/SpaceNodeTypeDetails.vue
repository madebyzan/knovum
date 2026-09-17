<script setup lang="ts">
import { ArrowRightLeft, MessageSquareQuote, ScanText, Sparkles } from "@lucide/vue";

import type { SpaceCanvasNode } from "../model/nodes/canvas-node";
import SpaceKnowledgeObjectDetails from "./knowledge/SpaceKnowledgeObjectDetails.vue";
import SpaceMediaKnowledge from "./media/SpaceMediaKnowledge.vue";
import SpaceDocumentCoverage from "./SpaceDocumentCoverage.vue";
import SpaceInspectorSection from "./SpaceInspectorSection.vue";
import SpacePortalDetails from "./SpacePortalDetails.vue";
import SpaceSubtitleDetails from "./SpaceSubtitleDetails.vue";
import SpaceVisualKnowledgeDetails from "./SpaceVisualKnowledgeDetails.vue";

defineProps<{ node: SpaceCanvasNode }>();
</script>

<template>
  <SpaceInspectorSection
    v-if="node.data.kind === 'file' && node.data.documentCoverage"
    :title="$t('spaces.processing.documentCoverage')"
    :icon="ScanText"
  >
    <SpaceDocumentCoverage :node="node" />
  </SpaceInspectorSection>
  <SpaceInspectorSection
    v-if="node.data.kind === 'object'"
    :title="$t('spaces.nodeTypes.object')"
    :icon="MessageSquareQuote"
  >
    <SpaceKnowledgeObjectDetails :node="node" />
  </SpaceInspectorSection>

  <SpaceInspectorSection
    v-if="node.data.kind === 'subtitle'"
    :title="$t('spaces.creation.subtitleEditor')"
    :icon="ScanText"
  >
    <SpaceSubtitleDetails :node="node" />
  </SpaceInspectorSection>

  <SpaceInspectorSection
    v-if="node.data.kind === 'audio' || node.data.kind === 'video'"
    :title="$t('spaces.media.knowledge')"
    :icon="ScanText"
  >
    <SpaceMediaKnowledge :node="node" />
  </SpaceInspectorSection>

  <SpaceInspectorSection
    v-if="node.data.kind === 'image' || node.data.kind === 'video'"
    :title="$t('spaces.knowledge.visualKnowledge')"
    :icon="Sparkles"
  >
    <SpaceVisualKnowledgeDetails :node="node" />
  </SpaceInspectorSection>

  <SpaceInspectorSection
    v-if="node.data.kind === 'portal'"
    :title="$t('spaces.nodeTypes.portal')"
    :icon="ArrowRightLeft"
  >
    <SpacePortalDetails :node="node" />
  </SpaceInspectorSection>
</template>
