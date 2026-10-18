<script setup lang="ts">
import { Play, Trash2 } from "@lucide/vue";
import { computed } from "vue";
import { useI18n } from "vue-i18n";

import UiButton from "../../../../shared/ui/UiButton.vue";
import UiIconButton from "../../../../shared/ui/UiIconButton.vue";
import UiInput from "../../../../shared/ui/UiInput.vue";
import UiSelect from "../../../../shared/ui/UiSelect.vue";
import UiTextarea from "../../../../shared/ui/UiTextarea.vue";
import { milliseconds, type VideoLineDraft } from "../video-dubbing-draft";

const line = defineModel<VideoLineDraft>({ required: true });
const props = defineProps<{
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
  <section class="space-y-2 border-t border-stroke-divider pt-3 pb-2">
    <div class="grid grid-cols-2 gap-2">
      <label class="block min-w-0 space-y-1 text-caption text-fg-secondary">
        {{ t("spaces.speech.videoDubbing.startSeconds") }}
        <UiInput
          :model-value="line.startSeconds"
          size="sm"
          type="number"
          min="0"
          step="0.001"
          :disabled="disabled"
          @update:model-value="update('startSeconds', $event)"
        />
      </label>
      <label class="block min-w-0 space-y-1 text-caption text-fg-secondary">
        {{ t("spaces.speech.videoDubbing.endSeconds") }}
        <UiInput
          :model-value="line.endSeconds"
          size="sm"
          type="number"
          min="0"
          step="0.001"
          :disabled="disabled"
          @update:model-value="update('endSeconds', $event)"
        />
      </label>
    </div>
    <UiTextarea
      :model-value="line.text"
      :rows="3"
      :disabled="disabled"
      :placeholder="t('spaces.speech.project.script.placeholder')"
      :aria-label="t('spaces.speech.project.script.label')"
      @update:model-value="update('text', $event)"
    />
    <p v-if="showSource && line.sourceText" class="text-caption text-fg-tertiary">
      {{ t("spaces.speech.translation.source") }}: {{ line.sourceText }}
    </p>
    <UiButton
      size="sm"
      :disabled="
        disabled ||
        !Number.isFinite(milliseconds(line.startSeconds)) ||
        milliseconds(line.endSeconds) <= milliseconds(line.startSeconds)
      "
      @click="emit('source', milliseconds(line.startSeconds), milliseconds(line.endSeconds))"
      >{{ t("spaces.speech.videoDubbing.auditionSourceWindow") }}</UiButton
    >
    <div class="flex items-center gap-1">
      <UiInput
        :model-value="line.speakerKey"
        size="sm"
        class="min-w-0 flex-1"
        :placeholder="t('spaces.speech.project.role')"
        :disabled="disabled"
        @update:model-value="update('speakerKey', $event)"
      />
      <UiSelect
        :model-value="line.voiceRevisionId"
        size="sm"
        class="min-w-0 flex-1"
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
    <UiTextarea
      v-if="supportsDeliveryInstruction"
      :model-value="line.deliveryInstruction"
      :rows="2"
      :maxlength="500"
      :disabled="disabled"
      :placeholder="t('spaces.speech.project.deliveryInstruction.placeholder')"
      :aria-label="t('spaces.speech.project.deliveryInstruction.label')"
      @update:model-value="update('deliveryInstruction', $event)"
    />
  </section>
</template>
