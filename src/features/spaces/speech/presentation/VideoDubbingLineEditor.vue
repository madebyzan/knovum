<script setup lang="ts">
import { Play, Trash2, Volume2 } from "@lucide/vue";
import { computed } from "vue";
import { useI18n } from "vue-i18n";

import UiIconButton from "../../../../shared/ui/UiIconButton.vue";
import UiInput from "../../../../shared/ui/UiInput.vue";
import UiSelect from "../../../../shared/ui/UiSelect.vue";
import UiTextarea from "../../../../shared/ui/UiTextarea.vue";
import { milliseconds, type VideoLineDraft } from "../video-dubbing-draft";
import SpeechDeliveryInstructionField from "./SpeechDeliveryInstructionField.vue";

const TIME_INPUT_CLASS =
  "h-6 w-14 border-transparent bg-transparent px-1 text-center font-mono text-caption-sm tabular-nums shadow-none hover:border-transparent hover:bg-surface-control focus:border-stroke-focus focus:bg-surface-raised [appearance:textfield] [&::-webkit-inner-spin-button]:appearance-none [&::-webkit-outer-spin-button]:appearance-none";

const line = defineModel<VideoLineDraft>({ required: true });
const props = defineProps<{
  index?: number;
  voiceOptions: readonly { value: string; label: string }[];
  deliveryInstructionVoiceIds: ReadonlySet<string>;
  lexiconValid: boolean;
  showSource: boolean;
  disabled: boolean;
}>();
const emit = defineEmits<{
  preview: [text: string, voiceRevisionId: string, deliveryInstruction?: string];
  remove: [];
  source: [startMs: number, endMs: number];
}>();
const { t } = useI18n();
const supportsDeliveryInstruction = computed(() =>
  props.deliveryInstructionVoiceIds.has(line.value.voiceRevisionId),
);
const startMs = computed(() => milliseconds(line.value.startSeconds));
const endMs = computed(() => milliseconds(line.value.endSeconds));
const validWindow = computed(
  () => Number.isFinite(startMs.value) && Number.isFinite(endMs.value) && endMs.value > startMs.value,
);
const durationSeconds = computed(() => (validWindow.value ? (endMs.value - startMs.value) / 1000 : 0));

function update<K extends keyof VideoLineDraft>(field: K, value: VideoLineDraft[K]) {
  line.value = { ...line.value, [field]: value };
}
function selectVoice(voiceRevisionId: string) {
  line.value = {
    ...line.value,
    voiceRevisionId,
    deliveryInstruction: props.deliveryInstructionVoiceIds.has(voiceRevisionId)
      ? line.value.deliveryInstruction
      : "",
  };
}
</script>

<template>
  <section
    :class="[
      'rounded-md border bg-surface-raised transition-colors focus-within:border-stroke-focus',
      validWindow ? 'border-stroke-default' : 'border-fg-danger/50',
    ]"
  >
    <div class="flex items-center justify-between gap-1 border-b border-stroke-divider px-1.5 py-1">
      <div class="flex min-w-0 items-center gap-0.5">
        <span v-if="index !== undefined" class="px-1 font-mono text-caption-sm text-fg-tertiary tabular-nums">
          #{{ index + 1 }}
        </span>
        <UiInput
          :model-value="line.startSeconds"
          size="sm"
          type="number"
          min="0"
          step="0.001"
          :class="TIME_INPUT_CLASS"
          :aria-label="t('spaces.speech.videoDubbing.startSeconds')"
          :disabled="disabled"
          @update:model-value="update('startSeconds', $event)"
        />
        <span class="text-caption-sm text-fg-tertiary" aria-hidden="true">–</span>
        <UiInput
          :model-value="line.endSeconds"
          size="sm"
          type="number"
          min="0"
          step="0.001"
          :class="TIME_INPUT_CLASS"
          :aria-label="t('spaces.speech.videoDubbing.endSeconds')"
          :disabled="disabled"
          @update:model-value="update('endSeconds', $event)"
        />
        <span v-if="durationSeconds > 0" class="px-1 text-caption-sm text-fg-tertiary tabular-nums">
          {{ durationSeconds.toFixed(1) }}s
        </span>
      </div>
      <div class="flex shrink-0 items-center gap-0.5">
        <span v-if="line.text.trim()" class="px-1 text-caption-sm text-fg-tertiary tabular-nums">
          {{ line.text.trim().length }}
        </span>
        <UiIconButton
          size="sm"
          tooltip
          :label="t('spaces.speech.videoDubbing.auditionSourceWindow')"
          :disabled="disabled || !validWindow"
          @click="emit('source', startMs, endMs)"
          ><Volume2 class="size-3.5"
        /></UiIconButton>
        <UiIconButton
          size="sm"
          tooltip
          class="hover:text-fg-danger"
          :label="t('spaces.speech.videoDubbing.removeLine')"
          :disabled="disabled"
          @click="emit('remove')"
          ><Trash2 class="size-3.5"
        /></UiIconButton>
      </div>
    </div>
    <UiTextarea
      class="border-0 bg-transparent px-2 py-1.5 shadow-none hover:border-0 focus:border-0"
      :model-value="line.text"
      :rows="2"
      :disabled="disabled"
      :placeholder="t('spaces.speech.project.script.placeholder')"
      :aria-label="t('spaces.speech.project.script.label')"
      @update:model-value="update('text', $event)"
    />
    <p
      v-if="showSource && line.sourceText"
      class="mx-2 mb-1.5 border-l-2 border-stroke-strong pl-2 text-caption-sm text-fg-tertiary"
    >
      {{ t("spaces.speech.translation.source") }}: {{ line.sourceText }}
    </p>
    <div class="flex items-center gap-1 border-t border-stroke-divider px-1.5 py-1">
      <UiInput
        :model-value="line.speakerKey"
        size="sm"
        class="h-6 w-20 shrink-0 border-transparent bg-transparent px-1.5 text-caption font-medium shadow-none hover:border-transparent hover:bg-surface-control focus:border-stroke-focus focus:bg-surface-raised"
        :placeholder="t('spaces.speech.project.role')"
        :disabled="disabled"
        @update:model-value="update('speakerKey', $event)"
      />
      <UiSelect
        :model-value="line.voiceRevisionId"
        size="sm"
        class="h-6 min-w-0 flex-1 border-transparent bg-transparent px-1.5 text-caption shadow-none hover:border-transparent hover:bg-surface-control focus:border-stroke-focus"
        :label="t('spaces.speech.voice.label')"
        :placeholder="t('spaces.speech.library.selection.placeholder')"
        :options="voiceOptions"
        :disabled="disabled"
        @update:model-value="selectVoice"
      />
      <UiIconButton
        size="sm"
        tooltip
        :label="t('spaces.speech.preview.action')"
        :disabled="disabled || !lexiconValid || !line.text.trim() || !line.voiceRevisionId"
        @click="
          emit(
            'preview',
            [...line.text.trim()].slice(0, 500).join(''),
            line.voiceRevisionId,
            line.deliveryInstruction.trim() || undefined,
          )
        "
        ><Play class="size-3.5"
      /></UiIconButton>
    </div>
    <div v-if="supportsDeliveryInstruction" class="px-2 pb-1.5">
      <SpeechDeliveryInstructionField
        collapsible
        surface="muted"
        :model-value="line.deliveryInstruction"
        :disabled="disabled"
        @update:model-value="update('deliveryInstruction', $event)"
      />
    </div>
  </section>
</template>
