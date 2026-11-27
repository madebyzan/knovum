<script setup lang="ts">
import { Plus } from "@lucide/vue";
import { computed, shallowRef, useTemplateRef, watch, watchEffect, type DeepReadonly } from "vue";
import { useI18n } from "vue-i18n";

import {
  LABEL_KEY_BY_TRANSLATION_TARGET,
  TRANSLATION_TARGETS,
  type TranslationTarget,
} from "../../../../foundation/ai/translation";
import UiButton from "../../../../shared/ui/UiButton.vue";
import type { SaveSpeechProjectRevisionRequest } from "../../contracts/generated/SaveSpeechProjectRevisionRequest";
import type { SpeechProjectSnapshot } from "../../contracts/generated/SpeechProjectSnapshot";
import type { VoiceLibraryRevision } from "../../contracts/generated/VoiceLibraryRevision";
import { useSpeechMediaSources } from "../use-speech-media-sources";
import { useVideoDubbingDraft } from "../use-video-dubbing-draft";
import { deliveryInstructionVoiceIds, voiceRevisionOptions } from "../voice-options";
import PronunciationLexiconEditor from "./PronunciationLexiconEditor.vue";
import SpeechEditorPage from "./SpeechEditorPage.vue";
import SpeechEditorTabs from "./SpeechEditorTabs.vue";
import type { SpeechSegmentedTabItem } from "./SpeechSegmentedTabs.vue";
import SpeechSettingsDisclosure from "./SpeechSettingsDisclosure.vue";
import VideoDubbingLines from "./VideoDubbingLines.vue";
import VideoDubbingMixSettings from "./VideoDubbingMixSettings.vue";
import VideoSourceControls from "./VideoSourceControls.vue";
import VideoTranslationControls from "./VideoTranslationControls.vue";

const props = defineProps<{
  project: DeepReadonly<SpeechProjectSnapshot>;
  voices: readonly VoiceLibraryRevision[];
  /** Commits the script on screen, which translating a revision requires. */
  save: () => Promise<boolean>;
  disabled: boolean;
}>();
const emit = defineEmits<{
  preview: [text: string, voiceId: string, lexicon: Record<string, string>, deliveryInstruction?: string];
  draftState: [
    value: boolean,
    input?: Omit<SaveSpeechProjectRevisionRequest, "projectId" | "expectedRevision">,
  ];
  busy: [value: boolean];
}>();
const { t } = useI18n();
const page = shallowRef("script");
const translationOpen = shallowRef(false);
const translationBusy = shallowRef(false);
const editingDisabled = computed(() => props.disabled || translationBusy.value);
watch(
  translationBusy,
  (value) => {
    if (value) translationOpen.value = true;
    emit("busy", value);
  },
  { immediate: true },
);
const sourceControls = useTemplateRef("sourceControls");
const draft = useVideoDubbingDraft(
  () => props.project,
  () => props.voices,
);
const pages = computed<SpeechSegmentedTabItem[]>(() => [
  {
    value: "script",
    label: t("spaces.speech.project.script.label"),
    badge: draft.lines.value.length || undefined,
  },
  {
    value: "mixing",
    label: t("spaces.speech.videoDubbing.mixingSettings"),
    invalid: !draft.lexicon.isValid.value,
  },
]);
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
    draft.initializeFromTranscript(transcript.value);
  },
  { immediate: true },
);
const voiceOptions = computed(() =>
  voiceRevisionOptions(
    props.voices,
    draft.lines.value.map((line) => line.voiceRevisionId),
  ),
);
const instructionVoiceIds = computed(() => deliveryInstructionVoiceIds(props.voices));
const translationSummary = computed(() => {
  const lang = draft.config.value?.translation?.targetLanguage;
  if (!lang || !TRANSLATION_TARGETS.includes(lang as TranslationTarget)) return undefined;
  return t(LABEL_KEY_BY_TRANSLATION_TARGET[lang as TranslationTarget]);
});
watchEffect(() =>
  emit(
    "draftState",
    draft.dirty.value,
    draft.needsRevision.value && draft.valid.value ? draft.revisionInput() : undefined,
  ),
);
function previewLine(text: string, voiceRevisionId: string, instruction?: string) {
  emit("preview", text, voiceRevisionId, draft.lexicon.serializeLexicon(), instruction);
}
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
        @apply-all="draft.applyVoiceToAll"
        @imported="draft.importSubtitles"
      />
      <SpeechSettingsDisclosure
        v-if="draft.lines.value.length"
        v-model:is-expanded="translationOpen"
        :title="t('spaces.speech.translation.translate')"
        :summary="translationSummary"
      >
        <VideoTranslationControls
          :project-id="project.project.id"
          :expected-revision="project.project.activeRevision"
          :line-count="draft.lines.value.length"
          :provenance="draft.config.value?.translation"
          :save="save"
          :disabled="editingDisabled"
          @translated="draft.applyTranslation"
          @busy="translationBusy = $event"
        />
      </SpeechSettingsDisclosure>
      <div
        v-if="!draft.lines.value.length"
        class="space-y-2.5 rounded-panel border border-dashed border-stroke-default bg-surface-panel p-4 text-center"
      >
        <p class="text-caption leading-relaxed text-fg-secondary">
          {{ t("spaces.speech.project.emptyVideoScript") }}
        </p>
        <div class="flex flex-wrap items-center justify-center gap-2">
          <UiButton
            v-if="transcript"
            size="sm"
            variant="secondary"
            :disabled="editingDisabled"
            @click="draft.importTranscript(transcript)"
          >
            {{ t("spaces.speech.videoDubbing.importTranscript") }}
          </UiButton>
          <UiButton size="sm" variant="ghost" :disabled="editingDisabled" @click="draft.addLine()"
            ><Plus class="size-3.5" />{{ t("spaces.speech.videoDubbing.addLine") }}</UiButton
          >
        </div>
      </div>
      <VideoDubbingLines
        :lines="draft.lines.value"
        :voice-options="voiceOptions"
        :delivery-instruction-voice-ids="instructionVoiceIds"
        :lexicon-valid="draft.lexicon.isValid.value"
        :show-source="Boolean(draft.config.value?.translation)"
        :disabled="editingDisabled"
        @preview="previewLine"
        @source="(start, end) => sourceControls?.audition(start, end)"
        @remove="draft.removeLine"
        @update="draft.updateLine"
      />
      <UiButton
        v-if="draft.lines.value.length"
        size="sm"
        variant="ghost"
        :disabled="editingDisabled"
        @click="draft.addLine()"
        ><Plus class="size-3.5" />{{ t("spaces.speech.videoDubbing.addLine") }}</UiButton
      >
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
        :summary="draft.lexicon.entries.value.length ? String(draft.lexicon.entries.value.length) : undefined"
        :is-invalid="!draft.lexicon.isValid.value"
      >
        <PronunciationLexiconEditor
          v-model="draft.lexicon.entries.value"
          :valid="draft.lexicon.isValid.value"
          :disabled="editingDisabled"
          hide-title
        />
      </SpeechSettingsDisclosure>
    </SpeechEditorPage>
  </SpeechEditorTabs>
</template>
