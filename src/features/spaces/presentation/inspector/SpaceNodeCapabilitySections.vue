<script setup lang="ts">
import { ArrowRightLeft, MessageSquareQuote, ScanText } from "@lucide/vue";

import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import SpaceDocumentCoverage from "../document/SpaceDocumentCoverage.vue";
import SpaceKnowledgeObjectInspection from "../knowledge/SpaceKnowledgeObjectInspection.vue";
import SpaceMediaKnowledge from "../media/SpaceMediaKnowledge.vue";
import SpaceNodeSpeechAction from "../speech/SpaceNodeSpeechAction.vue";
import SpaceInspectorSection from "./SpaceInspectorSection.vue";
import SpacePortalEditor from "./SpacePortalEditor.vue";

defineProps<{ node: SpaceCanvasNode }>();
</script>

<template>
  <SpaceNodeSpeechAction :node="node" />
  <SpaceInspectorSection
    v-if="node.data.kind === 'file' && node.data.documentCoverage"
    :title="$t('spaces.processing.document.coverage')"
    :icon="ScanText"
    :default-expanded="false"
  >
    <SpaceDocumentCoverage :node="node" />
  </SpaceInspectorSection>
  <SpaceInspectorSection
    v-if="node.data.kind === 'object'"
    :title="$t('spaces.nodeTypes.object')"
    :icon="MessageSquareQuote"
  >
    <SpaceKnowledgeObjectInspection :node-id="node.id" :data="node.data" />
  </SpaceInspectorSection>

  <SpaceInspectorSection
    v-if="node.data.kind === 'audio' || node.data.kind === 'video'"
    :title="$t('spaces.media.analysis.knowledge.title')"
    :icon="ScanText"
    :default-expanded="false"
  >
    <SpaceMediaKnowledge :node="node" />
  </SpaceInspectorSection>

  <SpaceInspectorSection
    v-if="node.data.kind === 'portal'"
    :title="$t('spaces.nodeTypes.portal')"
    :icon="ArrowRightLeft"
  >
    <SpacePortalEditor :node="node" />
  </SpaceInspectorSection>
</template>
