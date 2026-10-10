<script setup lang="ts">
import { Search, X } from "@lucide/vue";
import { useFocus } from "@vueuse/core";
import { type HTMLAttributes, useTemplateRef } from "vue";
import { useI18n } from "vue-i18n";

import { cn } from "./cn";
import { INPUT_BASE_CLASSES } from "./ui-classes";

const { t } = useI18n();
const model = defineModel<string>({ default: "" });

const props = withDefaults(
  defineProps<{
    label?: string | undefined;
    placeholder?: string | undefined;
    disabled?: boolean | undefined;
    autofocus?: boolean | undefined;
    size?: "sm" | "md" | undefined;
    class?: HTMLAttributes["class"];
  }>(),
  {
    placeholder: "",
    disabled: false,
    autofocus: false,
    size: "md",
  },
);

const emit = defineEmits<{
  escape: [];
}>();

const inputRef = useTemplateRef<HTMLInputElement>("input");
const { focused } = useFocus(inputRef, { initialValue: props.autofocus });

function handleEscape(event: KeyboardEvent) {
  if (event.isComposing) return;
  if (model.value) {
    model.value = "";
  } else {
    emit("escape");
  }
}

function clear() {
  model.value = "";
  focus();
}

function focus() {
  focused.value = true;
}

defineExpose({ focus, focused });
</script>

<template>
  <div :class="cn('relative flex w-full items-center', props.class)">
    <Search
      aria-hidden="true"
      :class="[
        'pointer-events-none absolute text-fg-tertiary',
        size === 'sm' ? 'left-2 size-3' : 'left-2.5 size-3.5',
      ]"
      :stroke-width="1.6"
    />
    <input
      ref="input"
      v-model="model"
      type="text"
      :aria-label="label"
      :placeholder="placeholder"
      :disabled="disabled"
      :autofocus="autofocus"
      :class="
        cn(
          'w-full bg-surface-muted/70',
          INPUT_BASE_CLASSES,
          size === 'sm'
            ? 'h-(--knovum-control-height-sm) pr-7 pl-7 text-caption'
            : 'h-(--knovum-control-height-md) pr-8 pl-8.5 text-label',
        )
      "
      @keydown.escape.stop="handleEscape"
    />
    <button
      v-if="model"
      type="button"
      :disabled="disabled"
      :aria-label="t('common.clearSearch')"
      :title="t('common.clearSearch')"
      :class="[
        'absolute flex items-center justify-center rounded-sm text-fg-tertiary transition-colors duration-100 hover:text-fg-primary focus-visible:outline-2 focus-visible:outline-stroke-focus disabled:pointer-events-none disabled:opacity-50',
        size === 'sm' ? 'right-0.5 size-6' : 'right-1 size-7',
      ]"
      @click="clear"
    >
      <X aria-hidden="true" :class="size === 'sm' ? 'size-2.5' : 'size-3'" :stroke-width="1.6" />
    </button>
  </div>
</template>
