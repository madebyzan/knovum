<script setup lang="ts">
import { Square, WandSparkles } from "@lucide/vue";
import { computed, shallowRef, type DeepReadonly } from "vue";
import { useI18n } from "vue-i18n";

import UiButton from "../../../../shared/ui/UiButton.vue";
import type { RunState } from "../../contracts/generated/RunState";
import type { SaveSpeechProjectRevisionRequest } from "../../contracts/generated/SaveSpeechProjectRevisionRequest";
import type { SpeechAudioOutput } from "../../contracts/generated/SpeechAudioOutput";
import type { SpeechGenerationProgress } from "../../contracts/generated/SpeechGenerationProgress";
import type { SpeechProjectSnapshot } from "../../contracts/generated/SpeechProjectSnapshot";
import type { VoiceLibraryRevision } from "../../contracts/generated/VoiceLibraryRevision";
import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import { nodeSpeechText } from "../../speech/node-speech";
import { workflowStateLabelKey } from "../../workflows/workflow-state-label";
import AudiobookScriptEditor from "./AudiobookScriptEditor.vue";
import ShortDubbingEditor from "./ShortDubbingEditor.vue";
import SpeechOutputPanel from "./SpeechOutputPanel.vue";
import VideoDeliveryPanel from "./VideoDeliveryPanel.vue";
import VideoDubbingEditor from "./VideoDubbingEditor.vue";

const props = defineProps<{
  sourceNode: SpaceCanvasNode;
  project: DeepReadonly<SpeechProjectSnapshot>;
  voices: readonly VoiceLibraryRevision[];
  preview?: (SpeechAudioOutput & { source: string }) | undefined;
  output?: SpeechAudioOutput | undefined;
  progress?: SpeechGenerationProgress | undefined;
  runState?: RunState | undefined;
  running: boolean;
  submitting: boolean;
  saving: boolean;
  cancelling: boolean;
  previewListening: boolean;
}>();
const emit = defineEmits<{
  save: [input: Omit<SaveSpeechProjectRevisionRequest, "projectId" | "expectedRevision">];
  preview: [
    text: string,
    voiceRevisionId: string,
    pronunciationLexicon: Record<string, string>,
    deliveryInstruction?: string,
  ];
  generate: [];
  cancel: [];
  stopListening: [];
  export: [path: string];
}>();
const { t } = useI18n();
const dirty = shallowRef(false);
const operationBusy = shallowRef(false);
const videoHasBackground = computed(
  () =>
    props.project.project.projectType === "video_dubbing" &&
    "background" in props.project.revision.outputConfig &&
    Boolean(props.project.revision.outputConfig.background),
);
const editorDisabled = computed(
  () => props.running || props.saving || props.submitting || operationBusy.value,
);
const canGenerate = computed(
  () =>
    !dirty.value &&
    !operationBusy.value &&
    !props.running &&
    !props.submitting &&
    props.project.segments.length > 0 &&
    props.project.segments.every((segment) => segment.state !== "needs_edit"),
);
const runLabel = computed(() => (props.runState ? t(workflowStateLabelKey(props.runState)) : undefined));

function stageLabel(stage: SpeechGenerationProgress["stage"]) {
  if (stage === "preparing") return t("spaces.speech.project.stages.preparing");
  if (stage === "synthesizing") return t("spaces.speech.project.stages.synthesizing");
  if (stage === "rendering") return t("spaces.speech.project.stages.rendering");
  return t("spaces.speech.project.stages.done");
}

function forwardPreview(
  text: string,
  voiceRevisionId: string,
  pronunciationLexicon: Record<string, string>,
  deliveryInstruction?: string,
) {
  emit("preview", text, voiceRevisionId, pronunciationLexicon, deliveryInstruction);
}
</script>

<template>
  <section class="space-y-3 rounded-md border border-stroke-default p-2.5">
    <div>
      <h2 class="truncate text-label font-medium text-fg-primary">{{ project.project.title }}</h2>
      <p class="text-caption-sm text-fg-tertiary">v{{ project.project.activeRevision }}</p>
    </div>
    <ShortDubbingEditor
      v-if="project.project.projectType === 'short_dubbing'"
      :project="project"
      :source-text="nodeSpeechText(sourceNode)"
      :voices="voices"
      :disabled="editorDisabled"
      @dirty="dirty = $event"
      @save="emit('save', $event)"
      @preview="forwardPreview"
    />
    <AudiobookScriptEditor
      v-else-if="project.project.projectType === 'audiobook'"
      :project="project"
      :source-node="sourceNode"
      :voices="voices"
      :disabled="editorDisabled"
      @dirty="dirty = $event"
      @save="emit('save', $event)"
      @preview="forwardPreview"
    />
    <VideoDubbingEditor
      v-else-if="project.project.projectType === 'video_dubbing'"
      :project="project"
      :voices="voices"
      :disabled="editorDisabled"
      @dirty="dirty = $event"
      @save="emit('save', $event)"
      @preview="forwardPreview"
      @busy="operationBusy = $event"
    />
    <div class="flex items-center gap-2 border-t border-stroke-divider pt-2">
      <UiButton size="sm" variant="primary" :disabled="!canGenerate" @click="emit('generate')">
        <WandSparkles class="size-3.5" />{{ t("spaces.speech.project.generate") }}
      </UiButton>
      <UiButton
        v-if="running"
        size="sm"
        variant="dangerSubtle"
        :disabled="cancelling"
        @click="emit('cancel')"
      >
        <Square class="size-3.5" />{{ t("common.cancel") }}
      </UiButton>
      <UiButton v-if="previewListening" size="sm" variant="ghost" @click="emit('stopListening')">
        <Square class="size-3.5" />{{ t("spaces.speech.project.stopListening") }}
      </UiButton>
      <span v-if="submitting || saving" class="text-caption-sm text-fg-tertiary">
        {{ t(submitting ? "spaces.speech.project.starting" : "spaces.speech.project.saving") }}
      </span>
    </div>
    <p v-if="runLabel && running" class="text-caption-sm text-fg-tertiary">{{ runLabel }}</p>
    <p v-if="progress && running" class="text-caption-sm text-fg-tertiary">
      {{ stageLabel(progress.stage) }} · {{ progress.completed }}/{{ progress.total }}
    </p>
    <audio
      v-if="preview && !dirty"
      class="h-8 w-full"
      controls
      :src="preview.source"
      @play="emit('stopListening')"
    />
    <VideoDeliveryPanel
      v-if="project.project.projectType === 'video_dubbing' && !dirty"
      :project-id="project.project.id"
      :revision-id="project.revision.id"
      :title="project.project.title"
      :generated="Boolean(output)"
      :has-background="videoHasBackground"
    />
    <SpeechOutputPanel
      v-if="output && !dirty"
      :project="project"
      :output="output"
      @export="emit('export', $event)"
    />
  </section>
</template>
