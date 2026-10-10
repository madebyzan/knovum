<script setup lang="ts">
import { useI18n } from "vue-i18n";

import UiCheckbox from "../../../../shared/ui/UiCheckbox.vue";
import UiInput from "../../../../shared/ui/UiInput.vue";
import type { SpeechMediaSource } from "../../contracts/generated/SpeechMediaSource";
import type { SpeechSourceSeparationPublication } from "../../contracts/generated/SpeechSourceSeparationPublication";
import type { VideoDubbingOutputConfig } from "../../contracts/generated/VideoDubbingOutputConfig";
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
      <label v-if="config.background" class="space-y-1 text-caption text-fg-secondary"
        >{{ t("spaces.speech.videoDubbing.backgroundGain") }}
        <UiInput v-model="backgroundGain" size="sm" inputmode="decimal" :disabled />
      </label>
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
        :disabled="disabled || !config.backgroundSeparation || !config.sourceDialogueIntervals.length"
        @update:model-value="updateMixSetting('preserveOriginalOutsideDialogue', $event)"
      />
      {{ t("spaces.speech.videoDubbing.preserveOriginalOutsideDialogue") }}
    </label>
  </div>
</template>
