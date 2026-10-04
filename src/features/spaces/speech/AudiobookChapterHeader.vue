<script setup lang="ts">
import { ArrowDown, ArrowUp, Trash2 } from "@lucide/vue";
import { computed } from "vue";
import { useI18n } from "vue-i18n";

import UiButton from "../../../shared/ui/UiButton.vue";
import UiInput from "../../../shared/ui/UiInput.vue";

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
      class="min-w-0 flex-1"
      size="sm"
      :placeholder="t('spaces.speech.chapterTitle')"
      :disabled="disabled"
    />
    <UiButton
      size="sm"
      variant="ghost"
      :aria-label="t('spaces.speech.moveUp')"
      :disabled="disabled || index === 0"
      @click="emit('move', -1)"
      ><ArrowUp class="size-3.5"
    /></UiButton>
    <UiButton
      size="sm"
      variant="ghost"
      :aria-label="t('spaces.speech.moveDown')"
      :disabled="disabled || index === count - 1"
      @click="emit('move', 1)"
      ><ArrowDown class="size-3.5"
    /></UiButton>
    <UiButton
      size="sm"
      variant="dangerSubtle"
      :aria-label="t('spaces.speech.removeChapter')"
      :disabled="disabled"
      @click="emit('remove')"
      ><Trash2 class="size-3.5"
    /></UiButton>
  </div>
</template>
