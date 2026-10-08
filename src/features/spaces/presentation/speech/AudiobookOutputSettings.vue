<script setup lang="ts">
import { useI18n } from "vue-i18n";

import UiInput from "../../../../shared/ui/UiInput.vue";
import UiSelect from "../../../../shared/ui/UiSelect.vue";
import type { WavPrecision } from "../../contracts/generated/WavPrecision";
import WavPrecisionSelect from "./WavPrecisionSelect.vue";

defineProps<{ disabled: boolean }>();
const format = defineModel<"wav" | "mp3" | "m4b">("format", { required: true });
const sampleRate = defineModel<"24000" | "48000">("sampleRate", { required: true });
const loudness = defineModel<string>("loudness", { required: true });
const peak = defineModel<string>("peak", { required: true });
const paragraphPause = defineModel<string>("paragraphPause", { required: true });
const chapterPause = defineModel<string>("chapterPause", { required: true });
const wavPrecision = defineModel<WavPrecision>("wavPrecision", { required: true });
const { t } = useI18n();
const formats = [
  { value: "m4b", label: "M4B" },
  { value: "mp3", label: "MP3" },
  { value: "wav", label: "WAV" },
] as const;
const sampleRates = [
  { value: "24000", label: "24 kHz" },
  { value: "48000", label: "48 kHz" },
] as const;
</script>

<template>
  <div class="grid grid-cols-2 gap-2">
    <UiSelect
      v-model="format"
      size="sm"
      :label="t('spaces.speech.project.format')"
      :options="formats"
      :disabled
    />
    <UiSelect
      v-model="sampleRate"
      size="sm"
      :label="t('spaces.speech.project.sampleRate')"
      :options="sampleRates"
      :disabled
    />
    <label class="text-caption text-fg-secondary">
      {{ t("spaces.speech.project.loudness") }}
      <UiInput v-model="loudness" size="sm" type="number" min="-24" max="-16" :disabled />
    </label>
    <label class="text-caption text-fg-secondary">
      {{ t("spaces.speech.project.truePeak") }}
      <UiInput v-model="peak" size="sm" type="number" min="-3" max="-1" :disabled />
    </label>
    <label class="text-caption text-fg-secondary"
      >{{ t("spaces.speech.audiobook.paragraphPause")
      }}<UiInput v-model="paragraphPause" size="sm" type="number" min="0" max="10000" :disabled
    /></label>
    <label class="text-caption text-fg-secondary"
      >{{ t("spaces.speech.audiobook.chapterPause")
      }}<UiInput v-model="chapterPause" size="sm" type="number" min="0" max="30000" :disabled
    /></label>
    <WavPrecisionSelect v-if="format === 'wav'" v-model="wavPrecision" :disabled />
  </div>
</template>
