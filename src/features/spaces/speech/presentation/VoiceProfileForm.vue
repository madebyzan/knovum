<script setup lang="ts">
import { useQuery } from "@tanstack/vue-query";
import { computed, shallowRef, watch } from "vue";
import { useI18n } from "vue-i18n";

import UiButton from "../../../../shared/ui/UiButton.vue";
import UiInput from "../../../../shared/ui/UiInput.vue";
import UiSelect from "../../../../shared/ui/UiSelect.vue";
import type { CreatePresetVoiceRequest } from "../../contracts/generated/CreatePresetVoiceRequest";
import { listPresetVoices } from "../ipc";
import { VOICE_QUERY_KEYS } from "../voice-query-keys";
import VoiceAudition from "./VoiceAudition.vue";

const emit = defineEmits<{
  create: [input: CreatePresetVoiceRequest];
  dirty: [value: boolean];
}>();
defineProps<{ disabled?: boolean }>();
const { t } = useI18n();
const { data: presets, error } = useQuery({
  queryKey: VOICE_QUERY_KEYS.presetCatalog,
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
const selectedPreset = computed(() => presets.value?.find((voice) => voice.speakerId === speakerId.value));
const identityDirty = shallowRef(false);
watch([language, presets], ([value]) => {
  const preferred = presets.value?.find((voice) => voice.primaryLanguage === value);
  if (preferred) speakerId.value = preferred.speakerId;
});
const LANGUAGE_OPTIONS = [
  { value: "zh-CN", label: "中文" },
  { value: "en", label: "English" },
] as const;

function submit() {
  const preset = selectedPreset.value;
  if (!preset) return;
  emit("create", {
    name: name.value.trim() || preset.name,
    presetVoiceId: speakerId.value,
    engineId: "moss_nano",
    modelSpecId: "moss-tts-nano-100m-onnx-bundled-v2",
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
  <form class="space-y-4" @submit.prevent="submit">
    <label class="block space-y-1.5 text-caption font-medium text-fg-secondary">
      <span>{{ t("spaces.speech.voice.name") }}</span>
      <UiInput
        v-model="name"
        size="sm"
        :placeholder="selectedPreset?.name"
        :aria-label="t('spaces.speech.voice.name')"
        :disabled="disabled"
        @update:model-value="markIdentityDirty"
      />
    </label>
    <p v-if="error" role="alert" class="text-caption text-fg-danger">{{ error.message }}</p>
    <div class="grid grid-cols-[minmax(0,1fr)_8rem] items-end gap-3">
      <label class="block min-w-0 space-y-1.5 text-caption font-medium text-fg-secondary">
        <span>{{ t("spaces.speech.voice.label") }}</span>
        <UiSelect
          v-model="speakerId"
          size="sm"
          :label="t('spaces.speech.voice.label')"
          :options="presetOptions"
          :disabled="disabled || !presets"
          @update:model-value="markIdentityDirty"
        />
      </label>
      <label class="block min-w-0 space-y-1.5 text-caption font-medium text-fg-secondary">
        <span>{{ t("spaces.speech.project.language") }}</span>
        <UiSelect
          v-model="language"
          size="sm"
          :label="t('spaces.speech.project.language')"
          :options="LANGUAGE_OPTIONS"
          :disabled="disabled"
          @update:model-value="markIdentityDirty"
        />
      </label>
    </div>
    <VoiceAudition
      :source="{ kind: 'preset', speakerId, language }"
      :disabled="Boolean(disabled) || !presets"
      @dirty="markAuditionDirty"
    >
      <template #actions>
        <UiButton size="sm" variant="primary" type="submit" :disabled="disabled || !presets">{{
          t("spaces.speech.voice.add")
        }}</UiButton>
      </template>
    </VoiceAudition>
  </form>
</template>
