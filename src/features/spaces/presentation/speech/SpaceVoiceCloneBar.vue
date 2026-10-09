<script setup lang="ts">
import { Mic2, Pause, Play, Scissors, SlidersHorizontal, X } from "@lucide/vue";
import { convertFileSrc } from "@tauri-apps/api/core";
import { ToolbarButton, ToolbarRoot, ToolbarSeparator } from "reka-ui";
import { shallowRef, toRef, useTemplateRef } from "vue";
import { useI18n } from "vue-i18n";

import UiActionHelp from "../../../../shared/ui/UiActionHelp.vue";
import UiButton from "../../../../shared/ui/UiButton.vue";
import UiIconButton from "../../../../shared/ui/UiIconButton.vue";
import UiInput from "../../../../shared/ui/UiInput.vue";
import UiPopover from "../../../../shared/ui/UiPopover.vue";
import { formatDurationMs } from "../../media/format-duration";
import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import { CANVAS_ACTION_BAR_CLASSES, CANVAS_ACTION_BUTTON_CLASSES } from "../canvas/canvas-action-classes";
import SpaceVoiceCloneAuditionControls from "./SpaceVoiceCloneAuditionControls.vue";
import SpeechMediaPlayer from "./SpeechMediaPlayer.vue";
import { useCanvasVoiceClone } from "./use-canvas-voice-clone";
import VoiceCandidateOptions from "./VoiceCandidateOptions.vue";
import VoiceReferenceRange from "./VoiceReferenceRange.vue";

const props = defineProps<{ node: SpaceCanvasNode; compact?: boolean }>();
const emit = defineEmits<{ close: [] }>();
const { t } = useI18n();
const { form, reference, setRange, save } = useCanvasVoiceClone(toRef(props, "node"));
const player = useTemplateRef("player");
const playing = shallowRef(false);
const playbackError = shallowRef<string>();
function auditionSelection() {
  if (playing.value) player.value?.pause();
  else void player.value?.seekTo(Number(reference.start) * 1000, Number(reference.end) * 1000);
}
</script>

<template>
  <ToolbarRoot
    :class="CANVAS_ACTION_BAR_CLASSES"
    :aria-label="t('spaces.speech.voice.clone')"
    :loop="false"
    data-testid="node-voice-clone-bar"
  >
    <Mic2 class="ml-2 size-4 text-fg-secondary" :stroke-width="1.6" />
    <span class="px-1.5 text-label font-medium" :class="compact && 'sr-only'">{{
      t("spaces.speech.voice.cloneShort")
    }}</span>
    <ToolbarSeparator class="mx-0.5 h-4 w-px bg-stroke-divider" />
    <ToolbarButton as-child
      ><UiIconButton
        :label="playing ? t('common.pause') : t('spaces.speech.reference.play')"
        size="sm"
        :disabled="!reference.selectionValid || form.busy"
        @click="auditionSelection"
        ><Pause v-if="playing" class="size-3.5" /><Play v-else class="size-3.5" /></UiIconButton
    ></ToolbarButton>
    <UiPopover align="start" class="w-72 space-y-3 p-3">
      <template #trigger
        ><ToolbarButton as-child
          ><button
            type="button"
            :class="CANVAS_ACTION_BUTTON_CLASSES"
            :disabled="form.busy || form.preparingReference"
            :aria-label="t('spaces.speech.reference.title')"
          >
            <Scissors class="size-4" :stroke-width="1.6" /><span class="tabular-nums"
              >{{ formatDurationMs(Number(reference.start) * 1000) }}–{{
                formatDurationMs(Number(reference.end) * 1000)
              }}</span
            >
          </button></ToolbarButton
        ></template
      >
      <VoiceReferenceRange
        :start="Number(reference.start)"
        :end="Number(reference.end)"
        :duration-ms="reference.durationMs ?? 0"
        :disabled="form.busy || form.preparingReference"
        @change="setRange"
      />
      <div class="grid grid-cols-2 gap-3">
        <label class="space-y-1 text-caption text-fg-secondary"
          ><span>{{ t("spaces.speech.reference.start") }}</span
          ><UiInput
            v-model="reference.start"
            size="sm"
            inputmode="decimal"
            :disabled="form.busy || form.preparingReference"
        /></label>
        <label class="space-y-1 text-caption text-fg-secondary"
          ><span>{{ t("spaces.speech.reference.end") }}</span
          ><UiInput
            v-model="reference.end"
            size="sm"
            inputmode="decimal"
            :disabled="form.busy || form.preparingReference"
        /></label>
      </div>
      <p
        class="text-caption-sm leading-relaxed"
        :class="reference.selectionValid ? 'text-fg-tertiary' : 'text-fg-danger'"
      >
        {{ t("spaces.speech.reference.requirements") }}
      </p>
    </UiPopover>
    <UiPopover class="w-72 p-3" align="start" force-mount>
      <template #trigger
        ><ToolbarButton as-child
          ><button
            type="button"
            :class="CANVAS_ACTION_BUTTON_CLASSES"
            :aria-label="t('spaces.speech.voice.options')"
          >
            <SlidersHorizontal class="size-4" :stroke-width="1.6" /></button></ToolbarButton
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
    <ToolbarSeparator class="mx-0.5 h-4 w-px bg-stroke-divider" />
    <UiButton v-if="reference.jobId" variant="ghost" size="sm" @click="reference.cancel">{{
      t("common.cancel")
    }}</UiButton>
    <UiButton
      v-else-if="!form.reference"
      variant="primary"
      size="sm"
      class="mx-1 bg-surface-inverse text-fg-inverse hover:bg-surface-inverse hover:opacity-90"
      :disabled="!reference.canPrepare"
      @click="reference.prepare"
      >{{ t("spaces.speech.reference.prepare") }}</UiButton
    >
    <SpaceVoiceCloneAuditionControls v-else :form="form" :save="save" />
    <UiActionHelp
      v-if="reference.error || playbackError || form.errors.length"
      :label="reference.error || playbackError || form.errors.join('\n')"
    />
    <ToolbarButton as-child
      ><UiIconButton :label="t('common.close')" size="sm" @click="emit('close')"
        ><X class="size-3.5" /></UiIconButton
    ></ToolbarButton>
  </ToolbarRoot>
  <SpeechMediaPlayer
    v-if="reference.path"
    :key="reference.sourceVersion"
    ref="player"
    class="hidden"
    kind="audio"
    :source="convertFileSrc(reference.path)"
    @loaded="reference.selectionLoaded"
    @error="playbackError = $event"
    @play="playing = true"
    @pause="playing = false"
    @ended="playing = false"
  />
</template>
