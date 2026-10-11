<script setup lang="ts">
import { Check, Pause, Play, Scissors, SlidersHorizontal } from "@lucide/vue";
import { convertFileSrc } from "@tauri-apps/api/core";
import { ToolbarButton } from "reka-ui";
import { shallowRef, useTemplateRef, watch } from "vue";
import { useI18n } from "vue-i18n";

import UiIconButton from "../../../../shared/ui/UiIconButton.vue";
import UiPopover from "../../../../shared/ui/UiPopover.vue";
import { formatDurationMs } from "../../media/format-duration";
import { CANVAS_ACTION_BUTTON_CLASSES } from "../canvas/canvas-action-classes";
import { NODE_ACTION_LABEL_CLASS, nodeActionLabelState } from "../canvas/node-action-label";
import SpeechMediaPlayer from "./SpeechMediaPlayer.vue";
import type { useCanvasVoiceClone } from "./use-canvas-voice-clone";
import VoiceCandidateOptions from "./VoiceCandidateOptions.vue";
import VoiceReferenceSelection from "./VoiceReferenceSelection.vue";

const props = defineProps<{
  form: ReturnType<typeof useCanvasVoiceClone>["form"];
  reference: ReturnType<typeof useCanvasVoiceClone>["reference"];
  compact: boolean;
}>();
const emit = defineEmits<{ playbackError: [message: string | undefined] }>();
const openPopover = defineModel<"range" | "options" | undefined>("openPopover", { required: true });
const { t } = useI18n();
const player = useTemplateRef("player");
const playing = shallowRef(false);
function setPopover(popover: "range" | "options", open: boolean) {
  openPopover.value = open ? popover : openPopover.value === popover ? undefined : openPopover.value;
}
watch([() => props.reference.start, () => props.reference.end], () => player.value?.pause());
watch([() => props.form.busy, () => props.form.preparingReference], ([busy, preparing]) => {
  if (busy || preparing) player.value?.pause();
});
function auditionSelection() {
  if (playing.value) player.value?.pause();
  else void player.value?.seekTo(Number(props.reference.start) * 1000, Number(props.reference.end) * 1000);
}
</script>

<template>
  <ToolbarButton as-child
    ><UiIconButton
      :label="playing ? t('common.pause') : t('spaces.speech.reference.play')"
      size="md"
      tooltip
      :disabled="!reference.selectionValid || form.busy"
      @click="auditionSelection"
      ><Pause v-if="playing" class="size-4.5" /><Play v-else class="size-4.5" /></UiIconButton
  ></ToolbarButton>
  <UiPopover
    :open="openPopover === 'range'"
    align="start"
    class="w-72 space-y-3 p-3"
    @update:open="setPopover('range', $event)"
  >
    <template #trigger
      ><ToolbarButton as-child
        ><button
          type="button"
          :class="CANVAS_ACTION_BUTTON_CLASSES"
          :disabled="form.busy || form.preparingReference"
          :aria-label="t('spaces.speech.reference.title')"
          :title="t('spaces.speech.reference.title')"
        >
          <component :is="form.reference ? Check : Scissors" class="size-4.5" :stroke-width="1.6" /><span
            class="tabular-nums"
            :class="[NODE_ACTION_LABEL_CLASS, nodeActionLabelState(compact)]"
            >{{ formatDurationMs(Number(reference.start) * 1000) }}–{{
              formatDurationMs(Number(reference.end) * 1000)
            }}</span
          >
        </button></ToolbarButton
      ></template
    >
    <VoiceReferenceSelection
      v-model:start="reference.start"
      v-model:end="reference.end"
      :duration-ms="reference.durationMs"
      :disabled="form.busy || form.preparingReference"
      :valid="reference.selectionValid"
    />
  </UiPopover>
  <UiPopover
    :open="openPopover === 'options'"
    class="w-80 p-3"
    align="start"
    force-mount
    @update:open="setPopover('options', $event)"
  >
    <template #trigger
      ><ToolbarButton as-child
        ><button
          type="button"
          :class="CANVAS_ACTION_BUTTON_CLASSES"
          :aria-label="t('spaces.speech.voice.options')"
          :title="t('spaces.speech.voice.options')"
          :disabled="form.busy || form.preparingReference"
        >
          <SlidersHorizontal class="size-4.5" :stroke-width="1.6" /></button></ToolbarButton
    ></template>
    <VoiceCandidateOptions
      v-model:mode="form.mode"
      v-model:model="form.model"
      v-model:available="form.modelAvailable"
      v-model:reference-transcript="form.referenceTranscript"
      v-model:language="form.language"
      fixed-source
      :disabled="form.busy || form.preparingReference"
      :modes="form.modes"
      :languages="form.languages"
    />
  </UiPopover>
  <SpeechMediaPlayer
    v-if="reference.path"
    :key="reference.sourceVersion"
    ref="player"
    class="hidden"
    kind="audio"
    :show-controls="false"
    :source="convertFileSrc(reference.path)"
    @loaded="reference.selectionLoaded"
    @error="emit('playbackError', $event)"
    @play="playing = true"
    @pause="playing = false"
    @ended="playing = false"
  />
</template>
