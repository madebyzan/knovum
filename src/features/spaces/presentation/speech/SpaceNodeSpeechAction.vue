<script setup lang="ts">
import { AudioLines } from "@lucide/vue";
import { Primitive, ToolbarButton } from "reka-ui";
import { computed, shallowRef, watch } from "vue";
import { useI18n } from "vue-i18n";

import UiButton from "../../../../shared/ui/UiButton.vue";
import UiDialog from "../../../../shared/ui/UiDialog.vue";
import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import { nodeSpeechProjectTypes } from "../../speech/node-speech";
import SpaceNodeSpeechEditor from "./SpaceNodeSpeechEditor.vue";
import VoiceCandidateForm from "./VoiceCandidateForm.vue";

const props = defineProps<{ node?: SpaceCanvasNode | undefined; compact?: boolean }>();
const { t } = useI18n();
const open = shallowRef(false);
const operations = computed(() => (props.node ? nodeSpeechProjectTypes(props.node) : []));
const audioSource = computed(() => (props.node?.data.kind === "audio" ? props.node : undefined));
const label = computed(() =>
  t(
    props.node?.data.kind === "audio"
      ? "spaces.speech.cloneVoice"
      : props.node?.data.kind === "video"
        ? "spaces.speech.types.videoDubbing"
        : props.node?.data.kind === "file"
          ? "spaces.speech.types.audiobook"
          : "spaces.speech.nodeReadAloud",
  ),
);
watch(
  () => props.node?.id,
  () => {
    open.value = false;
  },
);
</script>

<template>
  <component :is="compact ? ToolbarButton : Primitive" v-if="operations.length || audioSource" as-child>
    <UiButton
      size="sm"
      variant="ghost"
      :aria-label="label"
      data-testid="node-speech-action"
      @click="open = true"
    >
      <AudioLines class="size-3.5" :stroke-width="1.6" /><span v-if="!compact">{{ label }}</span>
    </UiButton>
  </component>
  <UiDialog v-if="node && open" v-model:open="open" :title="label" :description="node.data.title" size="wide">
    <VoiceCandidateForm v-if="audioSource" :key="node.id" :source-node="audioSource" />
    <SpaceNodeSpeechEditor v-else :key="node.id" :node="node" />
  </UiDialog>
</template>
