<script setup lang="ts">
import { useQuery } from "@tanstack/vue-query";
import { computed, shallowRef, watch } from "vue";
import { useI18n } from "vue-i18n";

import UiButton from "../../../../shared/ui/UiButton.vue";
import UiInput from "../../../../shared/ui/UiInput.vue";
import UiSelect from "../../../../shared/ui/UiSelect.vue";
import type { CreatePresetVoiceRequest } from "../../contracts/generated/CreatePresetVoiceRequest";
import { listPresetVoices } from "../../speech/ipc";
import { voiceQueryKeys } from "../../speech/voice-query-keys";
import VoiceAudition from "./VoiceAudition.vue";

const emit = defineEmits<{
  create: [input: CreatePresetVoiceRequest];
  dirty: [value: boolean];
}>();
defineProps<{ disabled?: boolean }>();
const { t } = useI18n();
const { data: presets, error } = useQuery({
  queryKey: voiceQueryKeys.presetCatalog,
  queryFn: listPresetVoices,
  staleTime: Infinity,
});
const presetOptions = computed(() =>
  (presets.value ?? [])
    .filter((voice) => voice.primaryLanguage === language.value)
    .map((voice) => ({
      value: voice.speakerId,
      label: `${voice.name} · ${voice.primaryLanguage} · ${t(`spaces.speech.voice.gender.${voice.gender}`)}`,
    })),
);
const name = shallowRef("");
const speakerId = shallowRef("0");
const language = shallowRef("zh-CN");
const identityDirty = shallowRef(false);
watch([language, presets], ([value]) => {
  const preferred = presets.value?.find((voice) => voice.primaryLanguage === value);
  if (preferred) speakerId.value = preferred.speakerId;
});
const languageOptions = [
  { value: "zh-CN", label: "中文" },
  { value: "en", label: "English" },
] as const;

function submit() {
  const preset = presets.value?.find((voice) => voice.speakerId === speakerId.value);
  if (!preset) return;
  emit("create", {
    name: name.value.trim() || preset.name,
    presetVoiceId: speakerId.value,
    engineId: "moss_nano",
    modelSpecId: "moss-tts-nano-onnx-v1",
    language: language.value,
    synthesisConfig: { speed: 1, sampleRate: 48_000, seed: 0 },
  });
}

function markIdentityDirty() {
  identityDirty.value = true;
  emit("dirty", true);
}

function markAuditionDirty(value: boolean) {
  emit("dirty", identityDirty.value || value);
}
</script>

<template>
  <form class="space-y-2" @submit.prevent="submit">
    <UiInput
      v-model="name"
      size="sm"
      :placeholder="t('spaces.speech.voice.name')"
      :aria-label="t('spaces.speech.voice.name')"
      :disabled="disabled"
      @update:model-value="markIdentityDirty"
    />
    <p v-if="error" class="text-caption text-fg-danger">{{ error.message }}</p>
    <div class="flex gap-2">
      <UiSelect
        v-model="speakerId"
        class="min-w-0 flex-1"
        size="sm"
        :label="t('spaces.speech.voice.label')"
        :options="presetOptions"
        :disabled="disabled || !presets"
        @update:model-value="markIdentityDirty"
      />
      <UiSelect
        v-model="language"
        class="min-w-28"
        size="sm"
        :label="t('spaces.speech.project.language')"
        :options="languageOptions"
        :disabled="disabled"
        @update:model-value="markIdentityDirty"
      />
      <UiButton size="sm" variant="primary" type="submit" :disabled="disabled || !presets">{{
        t("spaces.speech.voice.add")
      }}</UiButton>
    </div>
    <VoiceAudition
      :source="{ kind: 'preset', speakerId, language }"
      :disabled="Boolean(disabled) || !presets"
      @dirty="markAuditionDirty"
    />
  </form>
</template>
