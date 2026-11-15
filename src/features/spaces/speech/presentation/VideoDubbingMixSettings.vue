<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";

import UiButton from "../../../../shared/ui/UiButton.vue";
import UiCheckbox from "../../../../shared/ui/UiCheckbox.vue";
import UiInput from "../../../../shared/ui/UiInput.vue";
import UiSlider from "../../../../shared/ui/UiSlider.vue";
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
function clampedGain(raw: string): number {
  const parsed = Number(raw);
  return Number.isFinite(parsed) ? Math.min(6, Math.max(-30, parsed)) : 0;
}
function updateMixSetting(field: "retainOriginalTrack" | "preserveOriginalOutsideDialogue", value: boolean) {
  config.value = { ...config.value, [field]: value };
}
</script>

<template>
  <div class="divide-y divide-stroke-divider">
    <section class="space-y-1.5 pb-3">
      <span class="block text-caption-sm font-medium text-fg-secondary">
        {{ t("spaces.speech.videoDubbing.backgroundAudio") }}
      </span>
      <VideoBackgroundSelect
        :space-id="spaceId"
        :config
        :disabled
        @background="emit('background', $event)"
        @separated="emit('separated', $event)"
      />
    </section>

    <section class="space-y-2.5 py-3">
      <div class="space-y-1">
        <span class="block text-caption-sm font-medium text-fg-secondary">
          {{ t("spaces.speech.videoDubbing.voiceGain") }}
        </span>
        <div class="flex items-center gap-2">
          <UiSlider
            :model-value="clampedGain(voiceGain)"
            :label="t('spaces.speech.videoDubbing.voiceGain')"
            :min="-30"
            :max="6"
            :step="0.5"
            :disabled="disabled"
            class="min-w-0 flex-1"
            @update:model-value="voiceGain = String($event)"
          />
          <UiInput
            v-model="voiceGain"
            size="sm"
            inputmode="decimal"
            class="w-14 shrink-0 text-right tabular-nums"
            :aria-label="t('spaces.speech.videoDubbing.voiceGain')"
            :disabled="disabled"
          />
        </div>
      </div>

      <div v-if="config.background" class="space-y-1.5 pt-1">
        <span class="block text-caption-sm font-medium text-fg-secondary">
          {{ t("spaces.speech.videoDubbing.backgroundGain") }}
        </span>
        <div class="flex items-center gap-2">
          <UiSlider
            :model-value="clampedGain(backgroundGain)"
            :label="t('spaces.speech.videoDubbing.backgroundGain')"
            :min="-30"
            :max="6"
            :step="0.5"
            :disabled="disabled"
            class="min-w-0 flex-1"
            @update:model-value="backgroundGain = String($event)"
          />
          <UiInput
            v-model="backgroundGain"
            size="sm"
            inputmode="decimal"
            class="w-14 shrink-0 text-right tabular-nums"
            :aria-label="t('spaces.speech.videoDubbing.backgroundGain')"
            :disabled="disabled"
          />
        </div>
        <div class="flex flex-wrap items-center gap-1">
          <UiButton
            v-for="preset in backgroundPresets"
            :key="preset.gain"
            size="sm"
            :variant="Number(backgroundGain) === Number(preset.gain) ? 'secondary' : 'ghost'"
            :disabled="disabled"
            @click="backgroundGain = preset.gain"
          >
            {{ preset.label }}
          </UiButton>
        </div>
      </div>
    </section>

    <section class="space-y-2 pt-3">
      <label class="flex cursor-pointer items-center gap-2 text-caption text-fg-secondary">
        <UiCheckbox
          :model-value="config.retainOriginalTrack"
          :label="t('spaces.speech.videoDubbing.retainOriginalTrack')"
          :disabled="disabled"
          @update:model-value="updateMixSetting('retainOriginalTrack', $event)"
        />
        <span>{{ t("spaces.speech.videoDubbing.retainOriginalTrack") }}</span>
      </label>
      <label class="flex cursor-pointer items-center gap-2 text-caption text-fg-secondary">
        <UiCheckbox
          :model-value="config.preserveOriginalOutsideDialogue"
          :label="t('spaces.speech.videoDubbing.preserveOriginalOutsideDialogue')"
          :disabled="
            disabled || !usesOriginalVideoBackground(config) || !config.sourceDialogueIntervals.length
          "
          @update:model-value="updateMixSetting('preserveOriginalOutsideDialogue', $event)"
        />
        <span>{{ t("spaces.speech.videoDubbing.preserveOriginalOutsideDialogue") }}</span>
      </label>
    </section>
  </div>
</template>
