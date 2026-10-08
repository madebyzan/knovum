<script setup lang="ts">
import { useI18n } from "vue-i18n";

import UiSelect from "../../../../shared/ui/UiSelect.vue";
import type { WavPrecision } from "../../contracts/generated/WavPrecision";

defineProps<{ disabled?: boolean }>();
const model = defineModel<WavPrecision>({ required: true });
const { t } = useI18n();
const options = [
  { value: "pcm16", label: "16-bit PCM" },
  { value: "pcm24", label: "24-bit PCM" },
  { value: "float32", label: "32-bit float" },
] as const;
</script>

<template>
  <UiSelect v-model="model" size="sm" :label="t('spaces.speech.audiobook.wavPrecision')" :options :disabled />
</template>
