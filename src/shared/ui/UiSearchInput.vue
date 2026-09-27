<script setup lang="ts">
import { Search, X } from "@lucide/vue";
import { useFocus } from "@vueuse/core";
import { type HTMLAttributes, useTemplateRef } from "vue";

import { cn } from "./cn";
import { INPUT_BASE_CLASSES } from "./ui-classes";

const model = defineModel<string>({ default: "" });

const props = withDefaults(
  defineProps<{
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

function handleEscape() {
  if (model.value) {
    model.value = "";
  } else {
    emit("escape");
  }
}

function focus() {
  focused.value = true;
}

defineExpose({ focus, focused });
</script>

<template>
  <div :class="cn('relative flex w-full items-center', props.class)">
    <Search
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
      tabindex="-1"
      :class="[
        'absolute flex items-center justify-center text-fg-tertiary transition-colors duration-100 hover:text-fg-primary',
        size === 'sm' ? 'right-2 size-3' : 'right-2.5 size-4',
      ]"
      @click="model = ''"
    >
      <X :class="size === 'sm' ? 'size-2.5' : 'size-3'" :stroke-width="1.6" />
    </button>
  </div>
</template>
