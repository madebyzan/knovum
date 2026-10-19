<script setup lang="ts">
import { computed, provide, shallowRef, useTemplateRef, type DeepReadonly } from "vue";
import { useI18n } from "vue-i18n";
import { RouterLink } from "vue-router";

import UiScrollArea from "../../../../shared/ui/UiScrollArea.vue";
import type { SaveSpeechProjectRevisionRequest } from "../../contracts/generated/SaveSpeechProjectRevisionRequest";
import type { SpeechAudioOutput } from "../../contracts/generated/SpeechAudioOutput";
import type { SpeechGenerationProgress } from "../../contracts/generated/SpeechGenerationProgress";
import type { SpeechProjectSnapshot } from "../../contracts/generated/SpeechProjectSnapshot";
import type { VoiceLibraryRevision } from "../../contracts/generated/VoiceLibraryRevision";
import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import { narrationText } from "../narration-text";
import AudiobookScriptEditor from "./AudiobookScriptEditor.vue";
import ShortDubbingEditor from "./ShortDubbingEditor.vue";
import { SPEECH_EDITOR_VIEWPORT_KEY } from "./speech-editor-viewport";
import SpeechMediaPlayer from "./SpeechMediaPlayer.vue";
import SpeechProjectActions from "./SpeechProjectActions.vue";
import SpeechSettingsDisclosure from "./SpeechSettingsDisclosure.vue";
import SpeechSignalInspection from "./SpeechSignalInspection.vue";
import VideoDeliveryExport from "./VideoDeliveryExport.vue";
import VideoDubbingEditor from "./VideoDubbingEditor.vue";

const props = defineProps<{
  sourceNode: SpaceCanvasNode;
  project: DeepReadonly<SpeechProjectSnapshot>;
  voices: readonly VoiceLibraryRevision[];
  preview?: (SpeechAudioOutput & { source: string }) | undefined;
  output?: SpeechAudioOutput | undefined;
  progress?: SpeechGenerationProgress | undefined;
  /** An audition this editor is running: it is reported and cancelled here. */
  previewing: boolean;
  /** A Run the Task Center is carrying: it is reported and cancelled there. */
  generating: boolean;
  submitting: boolean;
  saving: boolean;
  cancelling: boolean;
  previewListening: boolean;
  /** Persists a draft as a new revision, for operations that run on a saved script. */
  save: (input: Omit<SaveSpeechProjectRevisionRequest, "projectId" | "expectedRevision">) => Promise<boolean>;
}>();
const emit = defineEmits<{
  preview: [
    text: string,
    voiceRevisionId: string,
    pronunciationLexicon: Record<string, string>,
    deliveryInstruction?: string,
  ];
  generate: [input?: Omit<SaveSpeechProjectRevisionRequest, "projectId" | "expectedRevision">];
  cancel: [];
  stopListening: [];
  draftState: [
    value: boolean,
    input?: Omit<SaveSpeechProjectRevisionRequest, "projectId" | "expectedRevision">,
  ];
}>();
const { t } = useI18n();
const scrollerRef = useTemplateRef("scrollerRef");
provide(
  SPEECH_EDITOR_VIEWPORT_KEY,
  computed(() => scrollerRef.value?.viewportElement ?? null),
);
const isDirty = shallowRef(false);
const draftInput = shallowRef<Omit<SaveSpeechProjectRevisionRequest, "projectId" | "expectedRevision">>();
const isOperationBusy = shallowRef(false);
const hasVideoBackground = computed(
  () =>
    props.project.project.projectType === "video_dubbing" &&
    "background" in props.project.revision.outputConfig &&
    Boolean(props.project.revision.outputConfig.background),
);
/** Voices are managed once, in Settings; here we only say when there is none to choose. */
const hasVoice = computed(() => props.voices.some((voice) => voice.isActive && voice.archivedAt == null));
const editorDisabled = computed(
  () => props.previewing || props.generating || props.saving || props.submitting || isOperationBusy.value,
);
const canGenerate = computed(
  () =>
    !editorDisabled.value &&
    (Boolean(draftInput.value) || (!isDirty.value && props.project.segments.length > 0)),
);
function stageLabel(stage: SpeechGenerationProgress["stage"]) {
  if (stage === "preparing") return t("spaces.speech.project.stages.preparing");
  if (stage === "synthesizing") return t("spaces.speech.project.stages.synthesizing");
  if (stage === "rendering") return t("spaces.speech.project.stages.rendering");
  return t("spaces.speech.project.stages.done");
}

/** Commits what the editor is showing, so an operation can work on a saved revision. */
async function saveDraft() {
  const input = draftInput.value;
  if (!input) return !isDirty.value;
  return await props.save(input);
}

function updateDraft(
  value: boolean,
  input?: Omit<SaveSpeechProjectRevisionRequest, "projectId" | "expectedRevision">,
) {
  isDirty.value = value;
  draftInput.value = input;
  emit("draftState", value, input);
}
</script>

<template>
  <section class="flex min-h-0 flex-col">
    <UiScrollArea ref="scrollerRef" class="max-h-96 min-h-0" viewport-class="max-h-[inherit]">
      <div class="space-y-3 p-3">
        <p v-if="!hasVoice" class="text-caption text-fg-secondary" role="status">
          {{ t("spaces.speech.library.selection.empty") }}
          <RouterLink
            class="text-fg-accent underline"
            data-testid="manage-voices"
            :to="{ path: '/settings', query: { pane: 'voices' } }"
            >{{ t("spaces.speech.library.manage") }}</RouterLink
          >
        </p>
        <ShortDubbingEditor
          v-if="project.project.projectType === 'short_dubbing'"
          :project="project"
          :source-text="narrationText(sourceNode)"
          :voices="voices"
          :disabled="editorDisabled"
          @draft-state="updateDraft"
          @preview="(...args) => emit('preview', ...args)"
        />
        <AudiobookScriptEditor
          v-else-if="project.project.projectType === 'audiobook'"
          :project="project"
          :source-node="sourceNode"
          @busy="isOperationBusy = $event"
          :voices="voices"
          :disabled="editorDisabled"
          @draft-state="updateDraft"
          @preview="(...args) => emit('preview', ...args)"
        />
        <VideoDubbingEditor
          v-else-if="project.project.projectType === 'video_dubbing'"
          :project="project"
          :voices="voices"
          :save="saveDraft"
          :disabled="editorDisabled"
          @draft-state="updateDraft"
          @preview="(...args) => emit('preview', ...args)"
          @busy="isOperationBusy = $event"
        />
        <SpeechMediaPlayer
          v-if="preview && !isDirty"
          kind="audio"
          :source="preview.source"
          @play="emit('stopListening')"
        />
        <SpeechSettingsDisclosure
          v-if="project.project.projectType === 'video_dubbing' && project.segments.length && !isDirty"
          :title="t('spaces.speech.videoDubbing.delivery')"
        >
          <VideoDeliveryExport
            :project-id="project.project.id"
            :revision-id="project.revision.id"
            :title="project.project.title"
            :generated="Boolean(output)"
            :has-background="hasVideoBackground"
          />
        </SpeechSettingsDisclosure>
        <SpeechSettingsDisclosure v-if="output && !isDirty" :title="t('spaces.speech.signal.inspect')">
          <SpeechSignalInspection :project-id="project.project.id" :revision-id="project.revision.id" />
        </SpeechSettingsDisclosure>
      </div>
    </UiScrollArea>
    <div v-if="previewing || generating" class="shrink-0 px-3 pb-2" role="status">
      <p v-if="generating" class="text-caption-sm text-fg-tertiary">
        {{ t("spaces.speech.project.generatingInTasks") }}
      </p>
      <p v-else-if="progress" class="text-caption-sm text-fg-tertiary">
        {{ stageLabel(progress.stage) }} · {{ progress.completed }}/{{ progress.total }}
      </p>
    </div>
    <SpeechProjectActions
      :can-generate="canGenerate"
      :project-type="project.project.projectType"
      :is-previewing="previewing"
      :is-generating="generating"
      :is-submitting="submitting"
      :is-saving="saving"
      :is-cancelling="cancelling"
      :is-preview-listening="previewListening"
      @generate="emit('generate', draftInput)"
      @cancel="emit('cancel')"
      @stop-listening="emit('stopListening')"
    />
  </section>
</template>
