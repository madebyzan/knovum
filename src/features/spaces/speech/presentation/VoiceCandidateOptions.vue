<script setup lang="ts">
import { useI18n } from "vue-i18n";

import type { LocalModelReferenceDuration } from "../../../../shared/contracts/generated/LocalModelReferenceDuration";
import type { LocalModelReferenceTranscript } from "../../../../shared/contracts/generated/LocalModelReferenceTranscript";
import UiSelect from "../../../../shared/ui/UiSelect.vue";
import UiSwitch from "../../../../shared/ui/UiSwitch.vue";
import type { VoiceModel } from "../../contracts/generated/VoiceModel";
import VoiceCandidateModelSelect from "./VoiceCandidateModelSelect.vue";

const mode = defineModel<"reference" | "designed">("mode", { required: true });
const model = defineModel<VoiceModel>("model", { required: true });
const initialModel = defineModel<VoiceModel>("initialModel", { required: true });
const language = defineModel<string>("language", { required: true });
const available = defineModel<boolean>("available", { required: true });
const referenceTranscript = defineModel<LocalModelReferenceTranscript>("referenceTranscript", {
  required: true,
});
const referenceDuration = defineModel<LocalModelReferenceDuration | undefined>("referenceDuration");
const supportsDesign = defineModel<boolean>("supportsDesign", { default: false });
const isolateVoice = defineModel<boolean>("isolateVoice", { required: true });
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
    <div class="space-y-3">
      <VoiceCandidateModelSelect
        v-model="model"
        v-model:initial-model="initialModel"
        v-model:available="available"
        v-model:reference-transcript="referenceTranscript"
        v-model:reference-duration="referenceDuration"
        v-model:supports-design="supportsDesign"
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
      <div v-if="mode === 'reference'" class="flex items-start justify-between gap-3">
        <span class="min-w-0 space-y-0.5">
          <span class="block text-caption font-medium text-fg-secondary">
            {{ t("spaces.speech.voice.isolate.label") }}
          </span>
          <span class="block text-caption text-fg-tertiary">{{ t("spaces.speech.voice.isolate.hint") }}</span>
        </span>
        <UiSwitch
          v-model="isolateVoice"
          :label="t('spaces.speech.voice.isolate.label')"
          :disabled="disabled"
        />
      </div>
    </div>
  </div>
</template>
