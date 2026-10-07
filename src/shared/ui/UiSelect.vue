<script setup lang="ts" generic="Value extends string = string">
import { Check, ChevronDown } from "@lucide/vue";
import {
  SelectContent,
  SelectItem,
  SelectItemIndicator,
  SelectItemText,
  SelectPortal,
  SelectRoot,
  SelectTrigger,
  SelectValue,
  SelectViewport,
  useId,
} from "reka-ui";
import { computed, useAttrs, type HTMLAttributes } from "vue";

import { cn } from "./cn";
import { INPUT_BASE_CLASSES, POPUP_ANIMATE_CLASSES, POPOVER_SURFACE_CLASSES } from "./ui-classes";

export interface SelectOption<Value extends string = string> {
  value: Value;
  label: string;
  description?: string;
  disabled?: boolean;
}

defineOptions({ inheritAttrs: false });
const attrs = useAttrs();
const model = defineModel<Value>({ required: true });

const props = withDefaults(
  defineProps<{
    id?: string | undefined;
    label: string;
    options: readonly SelectOption<Value>[];
    disabled?: boolean | undefined;
    placeholder?: string | undefined;
    size?: "sm" | "md" | undefined;
    width?: "normal" | "compact" | undefined;
    class?: HTMLAttributes["class"];
  }>(),
  {
    id: undefined,
    disabled: false,
    placeholder: undefined,
    size: "md",
    width: "normal",
  },
);

const selectId = useId(props.id);
const selectedLabel = computed(() => props.options.find((option) => option.value === model.value)?.label);
</script>

<template>
  <SelectRoot v-model="model" :disabled="disabled">
    <SelectTrigger
      v-bind="attrs"
      :id="selectId"
      :aria-label="label"
      :class="
        cn(
          'flex items-center justify-between gap-2 text-label data-placeholder:text-fg-tertiary data-[state=open]:border-stroke-focus',
          width === 'compact' ? 'w-20 min-w-20' : 'w-full min-w-32',
          INPUT_BASE_CLASSES,
          size === 'sm'
            ? 'h-(--knovum-control-height-sm) px-2 text-caption'
            : 'h-(--knovum-control-height-md) px-2.5 text-label',
          props.class,
        )
      "
    >
      <SelectValue v-bind="placeholder ? { placeholder } : {}" class="min-w-0 flex-1 truncate text-left">{{
        selectedLabel
      }}</SelectValue>
      <ChevronDown class="size-3 shrink-0 text-fg-tertiary" />
    </SelectTrigger>
    <SelectPortal>
      <SelectContent
        position="popper"
        :side-offset="6"
        :collision-padding="8"
        avoid-collisions
        :class="
          cn(
            POPUP_ANIMATE_CLASSES,
            POPOVER_SURFACE_CLASSES,
            'z-50 max-h-60 max-w-md min-w-(--reka-select-trigger-width) overflow-y-auto p-1 duration-100 ease-standard select-none',
          )
        "
      >
        <SelectViewport class="space-y-0.5">
          <SelectItem
            v-for="option in options"
            :key="option.value"
            :value="option.value"
            :data-value="option.value"
            :text-value="option.label"
            :disabled="option.disabled ?? false"
            class="relative flex min-h-interaction cursor-default items-center rounded-md py-1 pr-2 pl-7 text-label outline-none data-disabled:opacity-40 data-highlighted:bg-surface-control-hover"
          >
            <SelectItemIndicator class="absolute left-2"><Check class="size-3" /></SelectItemIndicator>
            <SelectItemText class="min-w-0 flex-1">
              <span class="block truncate">{{ option.label }}</span>
              <span v-if="option.description" class="block truncate text-caption-sm text-fg-tertiary">
                {{ option.description }}
              </span>
            </SelectItemText>
          </SelectItem>
        </SelectViewport>
      </SelectContent>
    </SelectPortal>
  </SelectRoot>
</template>
