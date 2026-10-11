<script setup lang="ts">
import { Plus } from "@lucide/vue";
import { computed, shallowRef, useTemplateRef, watch, watchEffect, type DeepReadonly } from "vue";
import { useI18n } from "vue-i18n";

import UiButton from "../../../../shared/ui/UiButton.vue";
import type { SaveSpeechProjectRevisionRequest } from "../../contracts/generated/SaveSpeechProjectRevisionRequest";
import type { SpeechProjectSnapshot } from "../../contracts/generated/SpeechProjectSnapshot";
import type { VoiceLibraryRevision } from "../../contracts/generated/VoiceLibraryRevision";
import { useSpeechMediaSources } from "../../speech/use-speech-media-sources";
import { useVideoDubbingDraft } from "../../speech/use-video-dubbing-draft";
import { voiceRevisionOptions } from "../../speech/voice-options";
import PronunciationLexiconEditor from "./PronunciationLexiconEditor.vue";
import SpeechEditorPage from "./SpeechEditorPage.vue";
import SpeechEditorTabs from "./SpeechEditorTabs.vue";
import SpeechSettingsDisclosure from "./SpeechSettingsDisclosure.vue";
import VideoDubbingLines from "./VideoDubbingLines.vue";
import VideoDubbingMixSettings from "./VideoDubbingMixSettings.vue";
import VideoSourceControls from "./VideoSourceControls.vue";
import VideoTranslationControls from "./VideoTranslationControls.vue";

const props = defineProps<{
  project: DeepReadonly<SpeechProjectSnapshot>;
  voices: readonly VoiceLibraryRevision[];
  disabled: boolean;
}>();
const emit = defineEmits<{
  preview: [
    text: string,
    voiceRevisionId: string,
    pronunciationLexicon: Record<string, string>,
    deliveryInstruction?: string,
  ];
  draftState: [
    value: boolean,
    input?: Omit<SaveSpeechProjectRevisionRequest, "projectId" | "expectedRevision">,
  ];
  busy: [value: boolean];
}>();
const { t } = useI18n();
const page = shallowRef("script");
const pages = computed(() => [
  { value: "script", label: t("spaces.speech.project.script.label") },
  { value: "mixing", label: t("spaces.speech.videoDubbing.mixingSettings") },
  { value: "translation", label: t("spaces.speech.translation.translate") },
]);
const translationBusy = shallowRef(false);
const editingDisabled = computed(() => props.disabled || translationBusy.value);
watch(translationBusy, (value) => emit("busy", value), { immediate: true });
const sourceControls = useTemplateRef("sourceControls");
const draft = useVideoDubbingDraft(
  () => props.project,
  () => props.voices,
);
const { videos } = useSpeechMediaSources();
const transcript = computed(
  () =>
    videos.value.find(
      (video) =>
        video.source.assetPath === draft.config.value?.video.assetPath &&
        video.source.sourceRevision === draft.config.value?.video.sourceRevision,
    )?.transcript,
);
watch(
  transcript,
  () => {
    if (
      props.project.project.activeRevision !== 1 ||
      draft.lines.value.length ||
      !transcript.value ||
      draft.dirty.value
    )
      return;
    draft.importTranscript(transcript.value, true);
  },
  { immediate: true },
);
const voiceOptions = computed(() =>
  voiceRevisionOptions(
    props.voices,
    draft.lines.value.map((line) => line.voiceRevisionId),
  ),
);
const deliveryInstructionVoiceIds = computed(
  () =>
    new Set(
      props.voices.filter((voice) => voice.supportsDeliveryInstruction).map((voice) => voice.revisionId),
    ),
);
watchEffect(() =>
  emit(
    "draftState",
    draft.dirty.value,
    draft.needsRevision.value && draft.valid.value ? draft.revisionInput() : undefined,
  ),
);
</script>

<template>
  <SpeechEditorTabs
    v-model="page"
    :items="pages"
    :label="t('spaces.speech.project.editorPages')"
    :invalid-page="!draft.lexicon.isValid.value ? 'mixing' : undefined"
  >
    <SpeechEditorPage value="script" :active="page">
      <VideoSourceControls
        v-if="draft.config.value"
        ref="sourceControls"
        v-model="draft.config.value"
        :lines="draft.lines.value"
        :voice-options="voiceOptions"
        :disabled="editingDisabled"
        @bind="draft.bindSpeaker"
        @imported="draft.importSubtitles"
      />
      <p v-if="!draft.lines.value.length" class="text-caption text-fg-secondary">
        {{ t("spaces.speech.project.emptyVideoScript") }}
      </p>
      <UiButton
        v-if="transcript && !draft.lines.value.length"
        size="sm"
        :disabled="editingDisabled"
        @click="draft.importTranscript(transcript)"
      >
        {{ t("spaces.speech.videoDubbing.importTranscript") }}
      </UiButton>
      <VideoDubbingLines
        :lines="draft.lines.value"
        :voice-options="voiceOptions"
        :delivery-instruction-voice-ids="deliveryInstructionVoiceIds"
        :lexicon-valid="draft.lexicon.isValid.value"
        :show-source="Boolean(draft.config.value?.translation)"
        :disabled="editingDisabled"
        @preview="
          (text, voiceRevisionId, instruction) =>
            emit('preview', text, voiceRevisionId, draft.lexicon.serializeLexicon(), instruction)
        "
        @source="(start, end) => sourceControls?.audition(start, end)"
        @remove="draft.removeLine"
        @update="draft.updateLine"
      />
      <div class="flex gap-2">
        <UiButton size="sm" variant="ghost" :disabled="editingDisabled" @click="draft.addLine()"
          ><Plus class="size-3.5" />{{ t("spaces.speech.videoDubbing.addLine") }}</UiButton
        >
      </div>
    </SpeechEditorPage>
    <SpeechEditorPage value="mixing" :active="page">
      <VideoDubbingMixSettings
        v-if="draft.config.value"
        v-model="draft.config.value"
        v-model:voice-gain="draft.voiceGain.value"
        v-model:background-gain="draft.backgroundGain.value"
        :space-id="project.project.spaceId"
        :disabled="editingDisabled"
        @background="draft.setBackground"
        @separated="draft.setSeparatedBackground"
      />
      <SpeechSettingsDisclosure
        :title="t('spaces.speech.audiobook.lexicon.title')"
        :is-invalid="!draft.lexicon.isValid.value"
      >
        <PronunciationLexiconEditor
          v-model="draft.lexicon.entries.value"
          :valid="draft.lexicon.isValid.value"
          :disabled="editingDisabled"
        />
      </SpeechSettingsDisclosure>
    </SpeechEditorPage>
    <SpeechEditorPage value="translation" :active="page">
      <VideoTranslationControls
        :project-id="project.project.id"
        :expected-revision="project.project.activeRevision"
        :segments="draft.translationInputs()"
        :provenance="draft.config.value?.translation"
        :disabled="editingDisabled"
        @translated="draft.applyTranslation"
        @busy="translationBusy = $event"
      />
    </SpeechEditorPage>
  </SpeechEditorTabs>
</template>
