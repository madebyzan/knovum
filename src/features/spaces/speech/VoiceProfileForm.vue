<script setup lang="ts">
import { shallowRef } from "vue";
import { useI18n } from "vue-i18n";

import UiButton from "../../../shared/ui/UiButton.vue";
import UiInput from "../../../shared/ui/UiInput.vue";
import UiSelect from "../../../shared/ui/UiSelect.vue";
import type { CreatePresetVoiceInput } from "./contracts";

const emit = defineEmits<{ create: [input: Omit<CreatePresetVoiceInput, "spaceId">] }>();
const { t } = useI18n();
const name = shallowRef("");
const speakerId = shallowRef("0");
const language = shallowRef("zh-CN");
const languageOptions = [
  { value: "zh-CN", label: "中文" },
  { value: "en", label: "English" },
] as const;

function submit() {
  if (!name.value.trim() || !/^\d+$/.test(speakerId.value)) return;
  emit("create", {
    name: name.value.trim(),
    presetVoiceId: speakerId.value,
    engineId: "kokoro",
    modelSpecId: "kokoro-int8-multi-lang-v1_0",
    language: language.value,
    synthesisConfig: { speed: 1, sampleRate: 24_000 },
  });
  name.value = "";
}
</script>

<template>
  <form class="space-y-2" @submit.prevent="submit">
    <UiInput v-model="name" size="sm" :placeholder="t('spaces.speech.voiceName')" />
    <div class="flex gap-2">
      <UiInput v-model="speakerId" class="min-w-0" size="sm" :placeholder="t('spaces.speech.speakerId')" />
      <UiSelect
        v-model="language"
        class="min-w-28"
        size="sm"
        :label="t('spaces.speech.language')"
        :options="languageOptions"
      />
      <UiButton size="sm" variant="primary" type="submit">{{ t("spaces.speech.addVoice") }}</UiButton>
    </div>
  </form>
</template>
