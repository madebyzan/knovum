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
    rows?: number | undefined;
    maxlength?: number | undefined;
    class?: HTMLAttributes["class"];
  }>(),
  {
    id: undefined,
    placeholder: "",
    disabled: false,
    rows: 4,
  },
);

const fieldId = useId(props.id);
const textareaRef = useTemplateRef<HTMLTextAreaElement>("textarea");

function focus() {
  textareaRef.value?.focus();
}

defineExpose({ focus });
</script>

<template>
  <textarea
    ref="textarea"
    v-model="model"
    :id="fieldId"
    :placeholder="placeholder"
    :disabled="disabled"
    :rows="rows"
    :maxlength="maxlength"
    :class="cn('w-full resize-y px-2.5 py-2 text-body', INPUT_BASE_CLASSES, props.class)"
  />
</template>
