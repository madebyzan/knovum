<script setup lang="ts">
import { ChevronDown, ChevronUp, Search, X } from "@lucide/vue";
import { nextTick, onMounted, useTemplateRef } from "vue";
import { useI18n } from "vue-i18n";

import UiIconButton from "./UiIconButton.vue";

const modelValue = defineModel<string>({ required: true });

defineProps<{
  placeholder?: string | undefined;
  matchCurrent?: number | undefined;
  matchTotal?: number | undefined;
}>();

const emit = defineEmits<{
  next: [];
  prev: [];
  close: [];
}>();

const { t } = useI18n();
const inputRef = useTemplateRef<HTMLInputElement>("input");

function onKeyDown(event: KeyboardEvent) {
  if (event.key === "Escape") {
    event.preventDefault();
    emit("close");
    return;
  }
  if (event.key === "Enter") {
    event.preventDefault();
    if (event.shiftKey) emit("prev");
    else emit("next");
  }
}

onMounted(() => {
  void nextTick(() => inputRef.value?.focus());
});
</script>

<template>
  <div
    data-reader-ignore-selection
    class="flex h-8.5 shrink-0 items-center gap-1.5 border-b border-stroke-divider/70 bg-surface-raised px-3 select-none"
  >
    <Search class="size-3.5 shrink-0 text-fg-tertiary" />
    <input
      ref="input"
      v-model="modelValue"
      type="text"
      :placeholder="placeholder ?? t('spaces.inspector.reader.searchPlaceholder')"
      class="min-w-0 flex-1 bg-transparent text-caption text-fg-primary outline-none placeholder:text-fg-tertiary"
      @keydown="onKeyDown"
    />
    <span
      v-if="modelValue.trim() && matchTotal !== undefined"
      class="shrink-0 text-caption-sm text-fg-tertiary tabular-nums"
    >
      {{ matchTotal > 0 ? `${matchCurrent ?? 0} / ${matchTotal}` : "0 / 0" }}
    </span>
    <UiIconButton
      :label="t('spaces.inspector.pdfReader.previousPage')"
      size="sm"
      :disabled="!matchTotal"
      @click="emit('prev')"
    >
      <ChevronUp class="size-3.5" />
    </UiIconButton>
    <UiIconButton
      :label="t('spaces.inspector.pdfReader.nextPage')"
      size="sm"
      :disabled="!matchTotal"
      @click="emit('next')"
    >
      <ChevronDown class="size-3.5" />
    </UiIconButton>
    <UiIconButton :label="t('common.close')" size="sm" @click="emit('close')">
      <X class="size-3.5" />
    </UiIconButton>
  </div>
</template>
