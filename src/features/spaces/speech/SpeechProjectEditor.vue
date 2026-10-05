<script setup lang="ts">
import { Square, WandSparkles } from "@lucide/vue";
import { computed, shallowRef, type DeepReadonly } from "vue";
import { useI18n } from "vue-i18n";

import UiButton from "../../../shared/ui/UiButton.vue";
import type { RunState } from "../contracts/generated/RunState";
import { workflowStateLabelKey } from "../workflows/workflow-state-label";
import AudiobookScriptEditor from "./AudiobookScriptEditor.vue";
import type {
  SaveSpeechProjectRevisionInput,
  SpeechAudioOutput,
  SpeechGenerationProgress,
  SpeechProjectDetail,
  VoiceLibraryRevision,
} from "./contracts";
import ShortDubbingEditor from "./ShortDubbingEditor.vue";
import SpeechOutputPanel from "./SpeechOutputPanel.vue";
import VideoDeliveryPanel from "./VideoDeliveryPanel.vue";
import VideoDubbingEditor from "./VideoDubbingEditor.vue";

const props = defineProps<{
  project: DeepReadonly<SpeechProjectDetail>;
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
  save: [input: Omit<SaveSpeechProjectRevisionInput, "projectId" | "expectedRevision">];
  preview: [text: string, voiceRevisionId: string, pronunciationLexicon: Record<string, string>];
  generate: [];
  cancel: [];
  stopListening: [];
  export: [path: string];
}>();
const { t } = useI18n();
const dirty = shallowRef(false);
const videoHasBackground = computed(
  () =>
    props.project.project.projectType === "video_dubbing" &&
    Boolean((JSON.parse(props.project.revision.outputConfig) as { background: unknown }).background),
);
const editorDisabled = computed(() => props.running || props.saving || props.submitting);
const canGenerate = computed(
  () =>
    !dirty.value &&
    !props.running &&
    !props.submitting &&
    props.project.segments.length > 0 &&
    props.project.segments.every((segment) => segment.state !== "needs_edit"),
);
const runLabel = computed(() => (props.runState ? t(workflowStateLabelKey(props.runState)) : undefined));

function stageLabel(stage: SpeechGenerationProgress["stage"]) {
  if (stage === "preparing") return t("spaces.speech.stages.preparing");
  if (stage === "synthesizing") return t("spaces.speech.stages.synthesizing");
  if (stage === "rendering") return t("spaces.speech.stages.rendering");
  return t("spaces.speech.stages.done");
}

function forwardPreview(text: string, voiceRevisionId: string, pronunciationLexicon: Record<string, string>) {
  emit("preview", text, voiceRevisionId, pronunciationLexicon);
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
      :voices="voices"
      :disabled="editorDisabled"
      @dirty="dirty = $event"
      @save="emit('save', $event)"
      @preview="forwardPreview"
    />
    <AudiobookScriptEditor
      v-else-if="project.project.projectType === 'audiobook'"
      :project="project"
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
    />
    <div class="flex items-center gap-2 border-t border-stroke-divider pt-2">
      <UiButton size="sm" variant="primary" :disabled="!canGenerate" @click="emit('generate')">
        <WandSparkles class="size-3.5" />{{ t("spaces.speech.generate") }}
      </UiButton>
      <UiButton
        v-if="running"
        size="sm"
        variant="dangerSubtle"
        :disabled="cancelling"
        @click="emit('cancel')"
      >
        <Square class="size-3.5" />{{ t("spaces.speech.cancel") }}
      </UiButton>
      <UiButton v-if="previewListening" size="sm" variant="ghost" @click="emit('stopListening')">
        <Square class="size-3.5" />{{ t("spaces.speech.stopListening") }}
      </UiButton>
      <span v-if="submitting || saving" class="text-caption-sm text-fg-tertiary">
        {{ t(submitting ? "spaces.speech.starting" : "spaces.speech.saving") }}
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
