<script setup lang="ts">
import { X } from "@lucide/vue";
import {
  TagsInputInput,
  TagsInputItem,
  TagsInputItemDelete,
  TagsInputItemText,
  TagsInputRoot,
  useId,
} from "reka-ui";
import { computed, type HTMLAttributes } from "vue";

import { cn } from "./cn";
import { INPUT_BASE_CLASSES } from "./ui-classes";

const model = defineModel<string[]>({ default: () => [] });

const props = withDefaults(
  defineProps<{
    id?: string | undefined;
    placeholder?: string | undefined;
    disabled?: boolean | undefined;
    max?: number | undefined;
    delimiter?: string | RegExp | undefined;
    addOnPaste?: boolean | undefined;
    addOnTab?: boolean | undefined;
    addOnBlur?: boolean | undefined;
    duplicate?: boolean | undefined;
    class?: HTMLAttributes["class"];
  }>(),
  {
    placeholder: "",
    disabled: false,
    addOnPaste: true,
    addOnTab: true,
    addOnBlur: true,
    duplicate: false,
  },
);

const fieldId = useId(props.id);

const rootProps = computed(() => ({
  id: fieldId,
  disabled: props.disabled,
  addOnPaste: props.addOnPaste,
  addOnTab: props.addOnTab,
  addOnBlur: props.addOnBlur,
  duplicate: props.duplicate,
  ...(props.max !== undefined ? { max: props.max } : {}),
  ...(props.delimiter !== undefined ? { delimiter: props.delimiter } : {}),
}));
</script>

<template>
  <TagsInputRoot
    v-bind="rootProps"
    v-model="model"
    :class="
      cn(
        'flex min-h-(--knovum-control-height-md) w-full flex-wrap items-center gap-1.5 px-2 py-1 text-label focus-within:border-focus-ring',
        INPUT_BASE_CLASSES,
        props.class,
      )
    "
  >
    <TagsInputItem
      v-for="(item, index) in model"
      :key="index"
      :value="item"
      class="inline-flex h-(--knovum-control-height-sm) items-center gap-1 rounded-control bg-control pr-1 pl-2 text-caption text-foreground select-none data-disabled:opacity-40"
    >
      <TagsInputItemText class="text-caption font-medium" />
      <TagsInputItemDelete
        type="button"
        class="grid size-4 place-items-center rounded-sm text-foreground-tertiary transition-colors duration-75 hover:bg-control-hover hover:text-foreground"
      >
        <X class="size-3" :stroke-width="1.6" />
      </TagsInputItemDelete>
    </TagsInputItem>

    <TagsInputInput
      :placeholder="model.length === 0 ? placeholder : ''"
      class="min-w-20 flex-1 bg-transparent text-label text-foreground outline-none placeholder:text-foreground-tertiary"
    />
  </TagsInputRoot>
</template>
