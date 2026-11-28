<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";

import type { useAudiobookDraft } from "../use-audiobook-draft";
import AudiobookMetadataEditor from "./AudiobookMetadataEditor.vue";
import AudiobookOutputSettings from "./AudiobookOutputSettings.vue";
import PronunciationLexiconEditor from "./PronunciationLexiconEditor.vue";
import SpeechSettingsDisclosure from "./SpeechSettingsDisclosure.vue";

const props = defineProps<{
  draft: ReturnType<typeof useAudiobookDraft>;
  disabled: boolean;
}>();
const { t } = useI18n();
const outputSummary = computed(
  () =>
    `${props.draft.output.format.value.toUpperCase()} · ${Number(props.draft.output.sampleRate.value) / 1000} kHz`,
);
const lexiconSummary = computed(() =>
  props.draft.lexicon.entries.value.length ? String(props.draft.lexicon.entries.value.length) : undefined,
);
const coverInvalid = computed(
  () => props.draft.output.format.value === "wav" && Boolean(props.draft.output.metadata.value.cover),
);
</script>

<template>
  <SpeechSettingsDisclosure :title="t('spaces.speech.audiobook.metadataSettings')" :is-expanded="true">
    <AudiobookMetadataEditor v-model="draft.output.metadata.value" :disabled="disabled" />
  </SpeechSettingsDisclosure>
  <SpeechSettingsDisclosure
    :title="t('spaces.speech.audiobook.outputSettings')"
    :summary="outputSummary"
    :is-invalid="coverInvalid"
  >
    <AudiobookOutputSettings
      v-model:format="draft.output.format.value"
      v-model:sample-rate="draft.output.sampleRate.value"
      v-model:loudness="draft.output.loudness.value"
      v-model:peak="draft.output.peak.value"
      v-model:paragraph-pause="draft.output.paragraphPause.value"
      v-model:chapter-pause="draft.output.chapterPause.value"
      v-model:wav-precision="draft.output.wavPrecision.value"
      :disabled="disabled"
    />
    <p v-if="coverInvalid" class="text-caption text-fg-danger">
      {{ t("spaces.speech.audiobook.coverRequiresEncodedOutput") }}
    </p>
  </SpeechSettingsDisclosure>
  <SpeechSettingsDisclosure
    :title="t('spaces.speech.audiobook.lexicon.title')"
    :summary="lexiconSummary"
    :is-invalid="!draft.lexicon.isValid.value"
  >
    <PronunciationLexiconEditor
      v-model="draft.lexicon.entries.value"
      :valid="draft.lexicon.isValid.value"
      :disabled="disabled"
      hide-title
    />
  </SpeechSettingsDisclosure>
</template>
