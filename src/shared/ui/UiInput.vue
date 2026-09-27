<script setup lang="ts">
import { useId } from "reka-ui";
import { type HTMLAttributes, useTemplateRef } from "vue";

import { cn } from "./cn";
import { INPUT_BASE_CLASSES } from "./ui-classes";

const model = defineModel<string>({ default: "" });

const props = withDefaults(
  defineProps<{
    id?: string | undefined;
    placeholder?: string | undefined;
    disabled?: boolean | undefined;
    type?: string | undefined;
    size?: "sm" | "md" | undefined;
    class?: HTMLAttributes["class"];
  }>(),
  {
    id: undefined,
    placeholder: "",
    disabled: false,
    type: "text",
    size: "md",
  },
);

const fieldId = useId(props.id);
const inputRef = useTemplateRef<HTMLInputElement>("input");

function focus() {
  inputRef.value?.focus();
}

defineExpose({ focus });
</script>

<template>
  <input
    ref="input"
    v-model="model"
    :id="fieldId"
    :type="type"
    :placeholder="placeholder"
    :disabled="disabled"
    :class="
      cn(
        'w-full',
        INPUT_BASE_CLASSES,
        size === 'sm'
          ? 'h-(--knovum-control-height-sm) px-2 text-caption'
          : 'h-(--knovum-control-height-md) px-2.5 text-label',
        props.class,
      )
    "
  />
</template>
