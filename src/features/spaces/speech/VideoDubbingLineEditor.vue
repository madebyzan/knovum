<script setup lang="ts">
import { Play, Trash2 } from "@lucide/vue";
import { useI18n } from "vue-i18n";

import UiButton from "../../../shared/ui/UiButton.vue";
import UiInput from "../../../shared/ui/UiInput.vue";
import UiSelect from "../../../shared/ui/UiSelect.vue";
import UiTextarea from "../../../shared/ui/UiTextarea.vue";
import { milliseconds, type VideoLineDraft } from "./video-dubbing-draft";

const line = defineModel<VideoLineDraft>({ required: true });
defineProps<{
  voiceOptions: readonly { value: string; label: string }[];
  lexiconValid: boolean;
  showSource: boolean;
  disabled: boolean;
}>();
const emit = defineEmits<{
  preview: [text: string, voiceRevisionId: string];
  remove: [];
  source: [startMs: number, endMs: number];
}>();
const { t } = useI18n();

function update<K extends keyof VideoLineDraft>(field: K, value: VideoLineDraft[K]) {
  line.value = { ...line.value, [field]: value };
}
</script>

<template>
  <section class="space-y-2 rounded-md border border-stroke-divider p-2">
    <div class="grid grid-cols-2 gap-2">
      <label class="text-caption text-fg-secondary">
        {{ t("spaces.speech.startSeconds") }}
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
      <label class="text-caption text-fg-secondary">
        {{ t("spaces.speech.endSeconds") }}
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
      :placeholder="t('spaces.speech.scriptPlaceholder')"
      @update:model-value="update('text', $event)"
    />
    <p v-if="showSource && line.sourceText" class="text-caption text-fg-tertiary">
      {{ t("spaces.speech.translationSource") }}: {{ line.sourceText }}
    </p>
    <UiButton
      size="sm"
      :disabled="
        disabled ||
        !Number.isFinite(milliseconds(line.startSeconds)) ||
        milliseconds(line.endSeconds) <= milliseconds(line.startSeconds)
      "
      @click="emit('source', milliseconds(line.startSeconds), milliseconds(line.endSeconds))"
      >{{ t("spaces.speech.auditionSourceWindow") }}</UiButton
    >
    <div class="flex items-center gap-1">
      <UiInput
        :model-value="line.speakerKey"
        size="sm"
        class="min-w-0 flex-1"
        :placeholder="t('spaces.speech.role')"
        :disabled="disabled"
        @update:model-value="update('speakerKey', $event)"
      />
      <UiSelect
        :model-value="line.voiceRevisionId"
        size="sm"
        class="min-w-0 flex-1"
        :label="t('spaces.speech.voice')"
        :options="voiceOptions"
        :disabled="disabled"
        @update:model-value="update('voiceRevisionId', $event)"
      />
      <UiButton
        size="sm"
        variant="ghost"
        :aria-label="t('spaces.speech.preview')"
        :disabled="disabled || !lexiconValid || !line.text.trim() || !line.voiceRevisionId"
        @click="emit('preview', [...line.text.trim()].slice(0, 500).join(''), line.voiceRevisionId)"
        ><Play class="size-3.5"
      /></UiButton>
      <UiButton
        size="sm"
        variant="dangerSubtle"
        :aria-label="t('spaces.speech.removeLine')"
        :disabled="disabled"
        @click="emit('remove')"
        ><Trash2 class="size-3.5"
      /></UiButton>
    </div>
    <p v-if="line.needsTimingEdit" class="text-caption text-fg-danger">
      {{ t("spaces.speech.needsTimingEdit") }}
    </p>
  </section>
</template>
