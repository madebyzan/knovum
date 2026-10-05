<script setup lang="ts">
import { Plus, Save } from "@lucide/vue";
import { computed, useTemplateRef, watch, type DeepReadonly } from "vue";
import { useI18n } from "vue-i18n";

import UiButton from "../../../shared/ui/UiButton.vue";
import UiInput from "../../../shared/ui/UiInput.vue";
import UiSelect from "../../../shared/ui/UiSelect.vue";
import type { SaveSpeechProjectRevisionInput, SpeechProjectDetail, VoiceLibraryRevision } from "./contracts";
import PronunciationLexiconEditor from "./PronunciationLexiconEditor.vue";
import { useSpeechMediaSources } from "./use-speech-media-sources";
import { useVideoDubbingDraft } from "./use-video-dubbing-draft";
import VideoDubbingLines from "./VideoDubbingLines.vue";
import VideoSourceControls from "./VideoSourceControls.vue";
import VideoSourceSeparationPanel from "./VideoSourceSeparationPanel.vue";
import VideoTranslationPanel from "./VideoTranslationPanel.vue";
import { voiceRevisionOptions } from "./voice-options";

const props = defineProps<{
  project: DeepReadonly<SpeechProjectDetail>;
  voices: readonly VoiceLibraryRevision[];
  disabled: boolean;
}>();
const emit = defineEmits<{
  save: [input: Omit<SaveSpeechProjectRevisionInput, "projectId" | "expectedRevision">];
  preview: [text: string, voiceRevisionId: string, pronunciationLexicon: Record<string, string>];
  dirty: [value: boolean];
}>();
const { t } = useI18n();
const sourceControls = useTemplateRef("sourceControls");
const draft = useVideoDubbingDraft(
  () => props.project,
  () => props.voices,
);
const { videos, backgrounds } = useSpeechMediaSources();
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
const backgroundValue = computed({
  get: () => {
    const current = draft.config.value?.background;
    if (!current) return "mute";
    return backgrounds.value.find(({ source }) => sameSource(source, current))?.id ?? "accepted-separation";
  },
  set: (value: string) => {
    if (value === "accepted-separation") return;
    draft.setBackground(
      value === "mute" ? null : (backgrounds.value.find(({ id }) => id === value)?.source ?? null),
    );
  },
});
const backgroundOptions = computed(() => [
  { value: "mute", label: t("spaces.speech.muteOriginal") },
  ...backgrounds.value.map(({ id, name }) => ({ value: id, label: name })),
  ...(backgroundValue.value === "accepted-separation"
    ? [{ value: "accepted-separation", label: t("spaces.speech.acceptedSeparatedBackground") }]
    : []),
]);
watch(draft.dirty, (value) => emit("dirty", value), { immediate: true });

function sameSource(
  left: { assetPath: string; sourceRevision: string },
  right: { assetPath: string; sourceRevision: string },
) {
  return left.assetPath === right.assetPath && left.sourceRevision === right.sourceRevision;
}
</script>

<template>
  <div class="space-y-3">
    <VideoSourceControls
      v-if="draft.config.value"
      ref="sourceControls"
      v-model="draft.config.value"
      :lines="draft.lines.value"
      :voice-options="voiceOptions"
      :disabled
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
          :disabled="disabled"
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
          :disabled="disabled"
        />
      </label>
    </div>
    <UiSelect
      v-model="backgroundValue"
      size="sm"
      :label="t('spaces.speech.backgroundAudio')"
      :options="backgroundOptions"
      :disabled="disabled"
    />
    <VideoSourceSeparationPanel
      v-if="draft.config.value"
      :space-id="project.project.spaceId"
      :source="draft.config.value.video"
      :disabled="disabled"
      @accepted="draft.setSeparatedBackground"
    />
    <PronunciationLexiconEditor
      v-model="draft.lexicon.entries.value"
      :valid="draft.lexicon.valid.value"
      :disabled="disabled"
    />
    <VideoTranslationPanel
      :project-id="project.project.id"
      :expected-revision="project.project.activeRevision"
      :segments="draft.translationInputs()"
      :provenance="draft.config.value?.translation"
      :disabled="disabled"
      @translated="draft.applyTranslation"
    />
    <UiButton
      v-if="transcript && !draft.lines.value.length"
      size="sm"
      :disabled="disabled"
      @click="draft.importTranscript(transcript)"
    >
      {{ t("spaces.speech.importTranscript") }}
    </UiButton>
    <VideoDubbingLines
      :lines="draft.lines.value"
      :voice-options="voiceOptions"
      :lexicon-valid="draft.lexicon.valid.value"
      :show-source="Boolean(draft.config.value?.translation)"
      :disabled
      @preview="(text, voiceRevisionId) => emit('preview', text, voiceRevisionId, draft.lexicon.value())"
      @source="(start, end) => sourceControls?.audition(start, end)"
      @remove="draft.removeLine"
      @update="draft.updateLine"
    />
    <div class="flex gap-2">
      <UiButton size="sm" :disabled="disabled" @click="draft.addLine()"
        ><Plus class="size-3.5" />{{ t("spaces.speech.addLine") }}</UiButton
      >
      <UiButton
        size="sm"
        :disabled="disabled || !draft.dirty.value || !draft.valid.value"
        @click="emit('save', draft.revisionInput())"
        ><Save class="size-3.5" />{{ t("spaces.speech.saveRevision") }}</UiButton
      >
    </div>
  </div>
</template>
