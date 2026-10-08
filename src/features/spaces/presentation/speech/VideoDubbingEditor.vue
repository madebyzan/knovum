<script setup lang="ts">
import { Plus, Save } from "@lucide/vue";
import { computed, shallowRef, useTemplateRef, watch, type DeepReadonly } from "vue";
import { useI18n } from "vue-i18n";

import UiButton from "../../../../shared/ui/UiButton.vue";
import UiInput from "../../../../shared/ui/UiInput.vue";
import type { SaveSpeechProjectRevisionRequest } from "../../contracts/generated/SaveSpeechProjectRevisionRequest";
import type { SpeechProjectSnapshot } from "../../contracts/generated/SpeechProjectSnapshot";
import type { VoiceLibraryRevision } from "../../contracts/generated/VoiceLibraryRevision";
import { useSpeechMediaSources } from "../../speech/use-speech-media-sources";
import { useVideoDubbingDraft } from "../../speech/use-video-dubbing-draft";
import { voiceRevisionOptions } from "../../speech/voice-options";
import PronunciationLexiconEditor from "./PronunciationLexiconEditor.vue";
import VideoBackgroundSelect from "./VideoBackgroundSelect.vue";
import VideoDubbingLines from "./VideoDubbingLines.vue";
import VideoSourceControls from "./VideoSourceControls.vue";
import VideoTranslationPanel from "./VideoTranslationPanel.vue";

const props = defineProps<{
  project: DeepReadonly<SpeechProjectSnapshot>;
  voices: readonly VoiceLibraryRevision[];
  disabled: boolean;
}>();
const emit = defineEmits<{
  save: [input: Omit<SaveSpeechProjectRevisionRequest, "projectId" | "expectedRevision">];
  preview: [
    text: string,
    voiceRevisionId: string,
    pronunciationLexicon: Record<string, string>,
    deliveryInstruction?: string,
  ];
  dirty: [value: boolean];
  busy: [value: boolean];
}>();
const { t } = useI18n();
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
watch(draft.dirty, (value) => emit("dirty", value), { immediate: true });
</script>

<template>
  <div class="space-y-3">
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
    <p class="text-caption text-fg-tertiary">{{ t("spaces.speech.videoTimingRule") }}</p>
    <div class="grid grid-cols-2 gap-2">
      <label class="text-caption text-fg-secondary"
        >{{ t("spaces.speech.voiceGain") }}
        <UiInput
          v-model="draft.voiceGain.value"
          size="sm"
          type="number"
          min="-30"
          max="6"
          :disabled="editingDisabled"
        />
      </label>
      <label v-if="draft.config.value?.background" class="text-caption text-fg-secondary"
        >{{ t("spaces.speech.backgroundGain") }}
        <UiInput
          v-model="draft.backgroundGain.value"
          size="sm"
          type="number"
          min="-30"
          max="6"
          :disabled="editingDisabled"
        />
      </label>
    </div>
    <VideoBackgroundSelect
      v-if="draft.config.value"
      :space-id="project.project.spaceId"
      :config="draft.config.value"
      :disabled="editingDisabled"
      @background="draft.setBackground"
      @separated="draft.setSeparatedBackground"
    />
    <PronunciationLexiconEditor
      v-model="draft.lexicon.entries.value"
      :valid="draft.lexicon.isValid.value"
      :disabled="editingDisabled"
    />
    <VideoTranslationPanel
      :project-id="project.project.id"
      :expected-revision="project.project.activeRevision"
      :segments="draft.translationInputs()"
      :provenance="draft.config.value?.translation"
      :disabled="editingDisabled"
      @translated="draft.applyTranslation"
      @busy="translationBusy = $event"
    />
    <UiButton
      v-if="transcript && !draft.lines.value.length"
      size="sm"
      :disabled="editingDisabled"
      @click="draft.importTranscript(transcript)"
    >
      {{ t("spaces.speech.importTranscript") }}
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
      <UiButton size="sm" :disabled="editingDisabled" @click="draft.addLine()"
        ><Plus class="size-3.5" />{{ t("spaces.speech.addLine") }}</UiButton
      >
      <UiButton
        size="sm"
        :disabled="editingDisabled || !draft.dirty.value || !draft.valid.value"
        @click="emit('save', draft.revisionInput())"
        ><Save class="size-3.5" />{{ t("spaces.speech.saveRevision") }}</UiButton
      >
    </div>
  </div>
</template>
