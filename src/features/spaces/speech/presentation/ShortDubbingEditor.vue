<script setup lang="ts">
import { Play } from "@lucide/vue";
import { computed, watchEffect, type DeepReadonly } from "vue";
import { useI18n } from "vue-i18n";

import UiButton from "../../../../shared/ui/UiButton.vue";
import UiSelect from "../../../../shared/ui/UiSelect.vue";
import UiTextarea from "../../../../shared/ui/UiTextarea.vue";
import type { SaveSpeechProjectRevisionRequest } from "../../contracts/generated/SaveSpeechProjectRevisionRequest";
import type { SpeechProjectSnapshot } from "../../contracts/generated/SpeechProjectSnapshot";
import type { VoiceLibraryRevision } from "../../contracts/generated/VoiceLibraryRevision";
import { useShortDubbingDraft } from "../use-short-dubbing-draft";
import { voiceRevisionOptions } from "../voice-options";
import PronunciationLexiconEditor from "./PronunciationLexiconEditor.vue";
import SpeechSettingsDisclosure from "./SpeechSettingsDisclosure.vue";
import WavPrecisionSelect from "./WavPrecisionSelect.vue";

const props = defineProps<{
  project: DeepReadonly<SpeechProjectSnapshot>;
  voices: readonly VoiceLibraryRevision[];
  disabled: boolean;
  sourceText: string;
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
}>();
const { t } = useI18n();
const {
  text,
  voiceRevisionId,
  deliveryInstruction,
  supportsDeliveryInstruction,
  wavPrecision,
  lexicon,
  dirty,
  needsRevision,
  valid,
  revisionInput,
} = useShortDubbingDraft(
  () => props.project,
  () => props.voices,
  () => props.sourceText,
);
const voiceOptions = computed(() => voiceRevisionOptions(props.voices, [voiceRevisionId.value]));

function preview() {
  if (!valid.value) return;
  emit(
    "preview",
    [...text.value.trim()].slice(0, 500).join(""),
    voiceRevisionId.value,
    lexicon.serializeLexicon(),
    deliveryInstruction.value.trim() || undefined,
  );
}

watchEffect(() =>
  emit("draftState", dirty.value, needsRevision.value && valid.value ? revisionInput() : undefined),
);
</script>

<template>
  <div class="space-y-3">
    <p class="text-caption font-medium text-fg-secondary">{{ t("spaces.speech.project.script.label") }}</p>
    <UiTextarea
      v-model="text"
      :rows="6"
      :disabled="disabled"
      :placeholder="t('spaces.speech.project.script.placeholder')"
      :aria-label="t('spaces.speech.project.script.label')"
    />
    <label v-if="voiceOptions.length" class="block space-y-1.5 text-caption font-medium text-fg-secondary">
      <span>{{ t("spaces.speech.voice.label") }}</span>
      <UiSelect
        v-model="voiceRevisionId"
        size="sm"
        :label="t('spaces.speech.voice.label')"
        :options="voiceOptions"
        :placeholder="t('spaces.speech.library.selection.placeholder')"
        :disabled="disabled"
      />
    </label>
    <UiTextarea
      v-if="supportsDeliveryInstruction"
      v-model="deliveryInstruction"
      :rows="2"
      :maxlength="500"
      :disabled="disabled"
      :placeholder="t('spaces.speech.project.deliveryInstruction.placeholder')"
      :aria-label="t('spaces.speech.project.deliveryInstruction.label')"
    />
    <div v-if="voiceOptions.length" class="flex gap-2">
      <UiButton size="sm" :disabled="disabled || !valid" @click="preview">
        <Play class="size-3.5" />{{ t("spaces.speech.preview.action") }}
      </UiButton>
    </div>
    <SpeechSettingsDisclosure
      :title="t('spaces.speech.project.settings')"
      :is-invalid="!lexicon.isValid.value"
    >
      <WavPrecisionSelect v-model="wavPrecision" :disabled />
      <PronunciationLexiconEditor
        v-model="lexicon.entries.value"
        :valid="lexicon.isValid.value"
        :disabled="disabled"
      />
    </SpeechSettingsDisclosure>
  </div>
</template>
