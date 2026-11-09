<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";

import UiButton from "../../../../shared/ui/UiButton.vue";
import UiCheckbox from "../../../../shared/ui/UiCheckbox.vue";
import UiInput from "../../../../shared/ui/UiInput.vue";
import type { SpeechMediaSource } from "../../contracts/generated/SpeechMediaSource";
import type { SpeechSourceSeparationPublication } from "../../contracts/generated/SpeechSourceSeparationPublication";
import type { VideoDubbingOutputConfig } from "../../contracts/generated/VideoDubbingOutputConfig";
import { usesOriginalVideoBackground } from "../video-background-selection";
import VideoBackgroundSelect from "./VideoBackgroundSelect.vue";

const config = defineModel<VideoDubbingOutputConfig>({ required: true });
const voiceGain = defineModel<string>("voiceGain", { required: true });
const backgroundGain = defineModel<string>("backgroundGain", { required: true });
defineProps<{ spaceId: string; disabled: boolean }>();
const emit = defineEmits<{
  background: [source: SpeechMediaSource | null];
  separated: [publication: SpeechSourceSeparationPublication];
}>();
const { t } = useI18n();
const backgroundPresets = computed(() => [
  { gain: "0", label: t("spaces.speech.videoDubbing.backgroundPresetNatural") },
  { gain: "-6", label: t("spaces.speech.videoDubbing.backgroundPresetSoft") },
  { gain: "-12", label: t("spaces.speech.videoDubbing.backgroundPresetSubtle") },
]);
function updateMixSetting(field: "retainOriginalTrack" | "preserveOriginalOutsideDialogue", value: boolean) {
  config.value = { ...config.value, [field]: value };
}
</script>

<template>
  <div class="space-y-3">
    <div class="space-y-1 text-caption text-fg-secondary">
      <span>{{ t("spaces.speech.videoDubbing.backgroundAudio") }}</span>
      <VideoBackgroundSelect
        :space-id="spaceId"
        :config
        :disabled
        @background="emit('background', $event)"
        @separated="emit('separated', $event)"
      />
    </div>
    <div class="grid grid-cols-2 gap-2">
      <label class="block min-w-0 space-y-1 text-caption text-fg-secondary"
        >{{ t("spaces.speech.videoDubbing.voiceGain") }}
        <UiInput v-model="voiceGain" size="sm" inputmode="decimal" :disabled />
      </label>
      <label v-if="config.background" class="block min-w-0 space-y-1 text-caption text-fg-secondary"
        >{{ t("spaces.speech.videoDubbing.backgroundGain") }}
        <UiInput v-model="backgroundGain" size="sm" inputmode="decimal" :disabled />
      </label>
    </div>
    <div v-if="config.background" class="flex flex-wrap items-center gap-1.5">
      <UiButton
        v-for="preset in backgroundPresets"
        :key="preset.gain"
        size="sm"
        :variant="Number(backgroundGain) === Number(preset.gain) ? 'secondary' : 'ghost'"
        :disabled
        @click="backgroundGain = preset.gain"
      >
        {{ preset.label }}
      </UiButton>
    </div>
    <label class="flex items-center gap-2 text-caption text-fg-secondary">
      <UiCheckbox
        :model-value="config.retainOriginalTrack"
        :label="t('spaces.speech.videoDubbing.retainOriginalTrack')"
        :disabled
        @update:model-value="updateMixSetting('retainOriginalTrack', $event)"
      />
      {{ t("spaces.speech.videoDubbing.retainOriginalTrack") }}
    </label>
    <label class="flex items-center gap-2 text-caption text-fg-secondary">
      <UiCheckbox
        :model-value="config.preserveOriginalOutsideDialogue"
        :label="t('spaces.speech.videoDubbing.preserveOriginalOutsideDialogue')"
        :disabled="disabled || !usesOriginalVideoBackground(config) || !config.sourceDialogueIntervals.length"
        @update:model-value="updateMixSetting('preserveOriginalOutsideDialogue', $event)"
      />
      {{ t("spaces.speech.videoDubbing.preserveOriginalOutsideDialogue") }}
    </label>
  </div>
</template>
