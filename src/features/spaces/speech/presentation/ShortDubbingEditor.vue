<script setup lang="ts">
import { Play } from "@lucide/vue";
import { computed, watch, watchEffect, type DeepReadonly } from "vue";
import { useI18n } from "vue-i18n";

import type { NarrationPreparationMode } from "../../../../shared/contracts/generated/NarrationPreparationMode";
import type { TaskTarget } from "../../../../shared/contracts/generated/TaskTarget";
import UiButton from "../../../../shared/ui/UiButton.vue";
import UiSelect from "../../../../shared/ui/UiSelect.vue";
import UiTextarea from "../../../../shared/ui/UiTextarea.vue";
import type { SaveSpeechProjectRevisionRequest } from "../../contracts/generated/SaveSpeechProjectRevisionRequest";
import type { SpeechProjectSnapshot } from "../../contracts/generated/SpeechProjectSnapshot";
import type { VoiceLibraryRevision } from "../../contracts/generated/VoiceLibraryRevision";
import { formatEstimatedDuration } from "../spoken-duration";
import { useNarrationScriptPreparation } from "../use-narration-script-preparation";
import { useShortDubbingDraft } from "../use-short-dubbing-draft";
import { voiceRevisionOptions } from "../voice-options";
import PronunciationLexiconEditor from "./PronunciationLexiconEditor.vue";
import SpeechDeliveryInstructionField from "./SpeechDeliveryInstructionField.vue";
import SpeechNarrationPrepareMenu from "./SpeechNarrationPrepareMenu.vue";
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
  busy: [value: boolean];
  draftState: [
    value: boolean,
    input?: Omit<SaveSpeechProjectRevisionRequest, "projectId" | "expectedRevision">,
  ];
}>();
const { t } = useI18n();
const preparation = useNarrationScriptPreparation<string>();
watch(preparation.preparing, (value) => emit("busy", value), { immediate: true });
const editingDisabled = computed(() => props.disabled || preparation.preparing.value);
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
const characterCount = computed(() => [...text.value.trim()].length);
const estimatedDuration = computed(() => formatEstimatedDuration(text.value));
const PRECISION_LABEL: Record<typeof wavPrecision.value, string> = {
  pcm16: "16-bit",
  pcm24: "24-bit",
  float32: "32-bit",
};
const settingsSummary = computed(() => {
  const precision = `WAV · ${PRECISION_LABEL[wavPrecision.value]}`;
  return lexicon.entries.value.length ? `${precision} · ${lexicon.entries.value.length}` : precision;
});

function prepareScript(mode: NarrationPreparationMode, target?: TaskTarget) {
  const snapshot = text.value;
  void preparation.run({
    mode,
    sourceText: snapshot,
    target,
    snapshot,
    onPreview: (partial) => {
      text.value = partial;
    },
    onComplete: (cleaned) => {
      text.value = cleaned;
    },
    onRestore: (previous) => {
      text.value = previous;
    },
  });
}

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
    <div class="space-y-1.5">
      <div class="flex items-center justify-between gap-2">
        <span class="text-caption font-medium text-fg-secondary">{{
          t("spaces.speech.project.script.label")
        }}</span>
        <div class="flex items-center gap-1.5">
          <SpeechNarrationPrepareMenu
            :disabled="disabled"
            :empty="characterCount === 0"
            :preparing="preparation.preparing.value"
            :can-restore="preparation.canRestore.value"
            @prepare="prepareScript"
            @cancel="preparation.cancel()"
            @restore="preparation.restore((previous) => (text = previous))"
          />
          <span class="text-caption-sm text-fg-tertiary tabular-nums">
            {{ t("spaces.richtext.statistics.characters", { count: characterCount }, characterCount)
            }}<template v-if="estimatedDuration"> · {{ estimatedDuration }}</template>
          </span>
        </div>
      </div>
      <UiTextarea
        v-model="text"
        :rows="5"
        :disabled="editingDisabled"
        :placeholder="t('spaces.speech.project.script.placeholder')"
        :aria-label="t('spaces.speech.project.script.label')"
      />
    </div>
    <div v-if="voiceOptions.length" class="space-y-1.5">
      <label class="block text-caption font-medium text-fg-secondary">{{
        t("spaces.speech.voice.label")
      }}</label>
      <div class="flex items-center gap-1.5">
        <UiSelect
          v-model="voiceRevisionId"
          size="sm"
          class="min-w-0 flex-1"
          :label="t('spaces.speech.voice.label')"
          :options="voiceOptions"
          :placeholder="t('spaces.speech.library.selection.placeholder')"
          :disabled="disabled"
        />
        <UiButton
          size="sm"
          variant="secondary"
          class="shrink-0"
          :disabled="disabled || !valid"
          @click="preview"
        >
          <Play class="size-3.5" />{{ t("spaces.speech.preview.action") }}
        </UiButton>
      </div>
    </div>
    <SpeechDeliveryInstructionField
      v-if="supportsDeliveryInstruction"
      v-model="deliveryInstruction"
      collapsible
      :disabled="disabled"
    />
    <SpeechSettingsDisclosure
      :title="t('spaces.speech.project.settings')"
      :summary="settingsSummary"
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
