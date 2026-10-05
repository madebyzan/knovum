<script setup lang="ts">
import { useQuery } from "@tanstack/vue-query";
import { computed, shallowRef } from "vue";
import { useI18n } from "vue-i18n";

import UiButton from "../../../shared/ui/UiButton.vue";
import UiInput from "../../../shared/ui/UiInput.vue";
import UiSelect from "../../../shared/ui/UiSelect.vue";
import type { CreatePresetVoiceInput } from "./contracts";
import { listPresetVoices } from "./ipc";
import VoiceAudition from "./VoiceAudition.vue";

const emit = defineEmits<{ create: [input: Omit<CreatePresetVoiceInput, "spaceId">] }>();
defineProps<{ disabled?: boolean }>();
const { t } = useI18n();
const { data: presets, error } = useQuery({
  queryKey: ["speech-preset-voices"],
  queryFn: listPresetVoices,
  staleTime: Infinity,
});
const presetOptions = computed(() =>
  (presets.value ?? []).map((voice) => ({
    value: voice.speakerId,
    label: `${voice.name} · ${voice.nativeLanguage} · ${t(`spaces.speech.${voice.gender}`)}`,
  })),
);
const name = shallowRef("");
const speakerId = shallowRef("0");
const language = shallowRef("zh-CN");
const languageOptions = [
  { value: "zh-CN", label: "中文" },
  { value: "en", label: "English" },
] as const;

function submit() {
  if (!name.value.trim() || !presets.value?.some((voice) => voice.speakerId === speakerId.value)) return;
  emit("create", {
    name: name.value.trim(),
    presetVoiceId: speakerId.value,
    engineId: "kokoro",
    modelSpecId: "kokoro-int8-multi-lang-v1_0",
    language: language.value,
    synthesisConfig: { speed: 1, sampleRate: 24_000, seed: 0 },
  });
  name.value = "";
}
</script>

<template>
  <form class="space-y-2" @submit.prevent="submit">
    <UiInput v-model="name" size="sm" :placeholder="t('spaces.speech.voiceName')" :disabled="disabled" />
    <p v-if="error" class="text-caption text-fg-danger">{{ error.message }}</p>
    <div class="flex gap-2">
      <UiSelect
        v-model="speakerId"
        class="min-w-0 flex-1"
        size="sm"
        :label="t('spaces.speech.voice')"
        :options="presetOptions"
        :disabled="disabled || !presets"
      />
      <UiSelect
        v-model="language"
        class="min-w-28"
        size="sm"
        :label="t('spaces.speech.language')"
        :options="languageOptions"
        :disabled="disabled"
      />
      <UiButton size="sm" variant="primary" type="submit" :disabled="disabled">{{
        t("spaces.speech.addVoice")
      }}</UiButton>
    </div>
    <VoiceAudition
      :source="{ kind: 'preset', speakerId, language }"
      :disabled="Boolean(disabled) || !presets"
    />
  </form>
</template>
