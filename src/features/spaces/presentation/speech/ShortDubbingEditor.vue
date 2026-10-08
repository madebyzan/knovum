<script setup lang="ts">
import { Play, Save } from "@lucide/vue";
import { computed, watch, type DeepReadonly } from "vue";
import { useI18n } from "vue-i18n";

import UiButton from "../../../../shared/ui/UiButton.vue";
import UiSelect from "../../../../shared/ui/UiSelect.vue";
import UiTextarea from "../../../../shared/ui/UiTextarea.vue";
import type { SaveSpeechProjectRevisionRequest } from "../../contracts/generated/SaveSpeechProjectRevisionRequest";
import type { SpeechProjectSnapshot } from "../../contracts/generated/SpeechProjectSnapshot";
import type { VoiceLibraryRevision } from "../../contracts/generated/VoiceLibraryRevision";
import { useShortDubbingDraft } from "../../speech/use-short-dubbing-draft";
import { voiceRevisionOptions } from "../../speech/voice-options";
import PronunciationLexiconEditor from "./PronunciationLexiconEditor.vue";
import WavPrecisionSelect from "./WavPrecisionSelect.vue";

const props = defineProps<{
  project: DeepReadonly<SpeechProjectSnapshot>;
  voices: readonly VoiceLibraryRevision[];
  disabled: boolean;
  sourceText: string;
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

watch(dirty, (value) => emit("dirty", value), { immediate: true });
</script>

<template>
  <div class="space-y-2">
    <UiTextarea
      v-model="text"
      :rows="6"
      :disabled="disabled"
      :placeholder="t('spaces.speech.project.scriptPlaceholder')"
    />
    <UiSelect
      v-model="voiceRevisionId"
      size="sm"
      :label="t('spaces.speech.voice.label')"
      :options="voiceOptions"
      :disabled="disabled || voices.length === 0"
    />
    <UiTextarea
      v-if="supportsDeliveryInstruction"
      v-model="deliveryInstruction"
      :rows="2"
      :maxlength="500"
      :disabled="disabled"
      :placeholder="t('spaces.speech.project.deliveryInstructionPlaceholder')"
      :aria-label="t('spaces.speech.project.deliveryInstruction')"
    />
    <WavPrecisionSelect v-model="wavPrecision" :disabled />
    <PronunciationLexiconEditor
      v-model="lexicon.entries.value"
      :valid="lexicon.isValid.value"
      :disabled="disabled"
    />
    <div class="flex gap-2">
      <UiButton size="sm" :disabled="disabled || !valid" @click="preview">
        <Play class="size-3.5" />{{ t("spaces.speech.project.preview") }}
      </UiButton>
      <UiButton size="sm" :disabled="disabled || !dirty || !valid" @click="emit('save', revisionInput())">
        <Save class="size-3.5" />{{ t("spaces.speech.project.saveRevision") }}
      </UiButton>
    </div>
  </div>
</template>
