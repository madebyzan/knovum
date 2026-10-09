<script setup lang="ts">
import { useI18n } from "vue-i18n";

import type { LocalModelReferenceTranscript } from "../../../../shared/contracts/generated/LocalModelReferenceTranscript";
import UiSelect from "../../../../shared/ui/UiSelect.vue";
import type { VoiceModel } from "../../contracts/generated/VoiceModel";
import VoiceCandidateModelSelect from "./VoiceCandidateModelSelect.vue";

const mode = defineModel<"reference" | "designed">("mode", { required: true });
const model = defineModel<VoiceModel>("model", { required: true });
const language = defineModel<string>("language", { required: true });
const available = defineModel<boolean>("available", { required: true });
const referenceTranscript = defineModel<LocalModelReferenceTranscript>("referenceTranscript", {
  required: true,
});
defineProps<{
  fixedSource: boolean;
  disabled: boolean;
  modes: readonly { value: string; label: string }[];
  languages: readonly { value: string; label: string }[];
}>();
const { t } = useI18n();
</script>

<template>
  <div class="space-y-3">
    <label v-if="!fixedSource" class="block space-y-1.5 text-caption font-medium text-fg-secondary">
      <span>{{ t("spaces.speech.voice.mode") }}</span>
      <UiSelect
        v-model="mode"
        size="sm"
        :label="t('spaces.speech.voice.mode')"
        :options="modes"
        :disabled="disabled"
      />
    </label>
    <div class="grid grid-cols-[minmax(0,1fr)_7rem] items-start gap-3">
      <VoiceCandidateModelSelect
        v-model="model"
        v-model:available="available"
        v-model:reference-transcript="referenceTranscript"
        :source-kind="mode"
        :disabled="disabled"
      />
      <label class="block min-w-0 space-y-1.5 text-caption font-medium text-fg-secondary">
        <span>{{ t("spaces.speech.project.language") }}</span>
        <UiSelect
          v-model="language"
          size="sm"
          :label="t('spaces.speech.project.language')"
          :options="languages"
          :disabled="disabled"
        />
      </label>
    </div>
  </div>
</template>
