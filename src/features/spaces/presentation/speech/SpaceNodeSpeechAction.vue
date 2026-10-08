<script setup lang="ts">
import { BookOpen, Clapperboard, Mic2, Speech } from "@lucide/vue";
import { Primitive, ToolbarButton } from "reka-ui";
import { computed, shallowRef, watch } from "vue";
import { useI18n } from "vue-i18n";

import UiButton from "../../../../shared/ui/UiButton.vue";
import UiDialog from "../../../../shared/ui/UiDialog.vue";
import UiTooltip from "../../../../shared/ui/UiTooltip.vue";
import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import { useSpaceStore } from "../../session/space-store";
import { nodeSpeechProjectTypes, type NodeSpeechProjectType } from "../../speech/node-speech";
import SpaceNodeBackgroundAction from "./SpaceNodeBackgroundAction.vue";
import SpaceNodeSpeechEditor from "./SpaceNodeSpeechEditor.vue";
import VoiceCandidateForm from "./VoiceCandidateForm.vue";

const props = defineProps<{ node?: SpaceCanvasNode | undefined; compact?: boolean }>();
const { t } = useI18n();
const store = useSpaceStore();
const selectedOperation = shallowRef<NodeSpeechProjectType | "clone">();
const operationIcons = {
  short_dubbing: Speech,
  audiobook: BookOpen,
  video_dubbing: Clapperboard,
  clone: Mic2,
};
const open = computed({
  get: () => selectedOperation.value !== undefined,
  set: (value: boolean) => {
    if (!value) selectedOperation.value = undefined;
  },
});
const operations = computed(() => {
  if (!props.node) return [];
  const clone = { id: "clone" as const, label: t("spaces.speech.cloneVoice") };
  if (props.node.data.kind === "audio") return [clone];
  const projects = nodeSpeechProjectTypes(props.node).map((id) => ({
    id,
    label: t(
      id === "short_dubbing"
        ? "spaces.speech.nodeReadAloud"
        : `spaces.speech.types.${id === "audiobook" ? "audiobook" : "videoDubbing"}`,
    ),
  }));
  return props.node.data.kind === "video" ? [...projects, clone] : projects;
});
const label = computed(() => operations.value.find(({ id }) => id === selectedOperation.value)?.label ?? "");
watch([() => props.node?.id, () => props.node?.data.sourceRevision, () => store.activeSpaceId], () => {
  open.value = false;
});
</script>

<template>
  <component
    :is="compact ? ToolbarButton : Primitive"
    v-for="operation in operations"
    :key="operation.id"
    as-child
  >
    <UiTooltip :text="operation.label" :disabled="!compact" side="top">
      <UiButton
        size="sm"
        variant="ghost"
        :aria-label="operation.label"
        data-testid="node-speech-action"
        :data-operation="operation.id"
        @click="selectedOperation = operation.id"
      >
        <component :is="operationIcons[operation.id]" class="size-3.5" :stroke-width="1.6" />
        <span v-if="!compact">{{ operation.label }}</span>
      </UiButton>
    </UiTooltip>
  </component>
  <SpaceNodeBackgroundAction
    v-if="node && (node.data.kind === 'audio' || node.data.kind === 'video')"
    :node="node"
    :compact="compact"
  />
  <UiDialog
    v-if="node && selectedOperation"
    v-model:open="open"
    :title="label"
    :description="node.data.title"
    size="wide"
  >
    <VoiceCandidateForm
      v-if="selectedOperation === 'clone' && store.activeSpaceId"
      :key="node.id"
      :source-node="node"
      :space-id="store.activeSpaceId"
    />
    <SpaceNodeSpeechEditor
      v-else-if="selectedOperation !== 'clone'"
      :key="`${node.id}:${selectedOperation}`"
      :node="node"
      :project-type="selectedOperation"
    />
  </UiDialog>
</template>
