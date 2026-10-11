<script setup lang="ts">
import { ArrowDown, ArrowUp, Trash2 } from "@lucide/vue";
import { computed } from "vue";
import { useI18n } from "vue-i18n";

import UiIconButton from "../../../../shared/ui/UiIconButton.vue";
import UiInput from "../../../../shared/ui/UiInput.vue";

const props = defineProps<{
  title: string;
  index: number;
  count: number;
  disabled: boolean;
}>();
const emit = defineEmits<{
  "update:title": [value: string];
  move: [offset: -1 | 1];
  remove: [];
}>();
const { t } = useI18n();
const title = computed({
  get: () => props.title,
  set: (value) => emit("update:title", value),
});
</script>

<template>
  <div class="flex items-center gap-1">
    <UiInput
      v-model="title"
      class="min-w-0 flex-1 border-transparent bg-transparent font-medium shadow-none hover:bg-surface-control"
      size="sm"
      :placeholder="t('spaces.speech.audiobook.chapterTitle')"
      :aria-label="t('spaces.speech.audiobook.chapterTitle')"
      :disabled="disabled"
    />
    <UiIconButton
      v-if="count > 1"
      size="sm"
      tooltip
      :label="t('spaces.speech.audiobook.moveUp')"
      :disabled="disabled || index === 0"
      @click="emit('move', -1)"
      ><ArrowUp class="size-3.5"
    /></UiIconButton>
    <UiIconButton
      v-if="count > 1"
      size="sm"
      tooltip
      :label="t('spaces.speech.audiobook.moveDown')"
      :disabled="disabled || index === count - 1"
      @click="emit('move', 1)"
      ><ArrowDown class="size-3.5"
    /></UiIconButton>
    <UiIconButton
      size="sm"
      tooltip
      class="hover:text-fg-danger"
      :label="t('spaces.speech.audiobook.removeChapter')"
      :disabled="disabled"
      @click="emit('remove')"
      ><Trash2 class="size-3.5"
    /></UiIconButton>
  </div>
</template>
