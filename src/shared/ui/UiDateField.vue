<script setup lang="ts">
import { parseDate } from "@internationalized/date";
import type { DateValue } from "@internationalized/date";
import { DateFieldInput, DateFieldRoot, useId, useLocale } from "reka-ui";
import { computed, type HTMLAttributes } from "vue";

import { cn } from "./cn";
import { INPUT_BASE_CLASSES } from "./ui-classes";

const model = defineModel<string | undefined>();

const props = withDefaults(
  defineProps<{
    id?: string | undefined;
    disabled?: boolean | undefined;
    locale?: string | undefined;
    size?: "sm" | "md" | undefined;
    class?: HTMLAttributes["class"];
  }>(),
  {
    id: undefined,
    disabled: false,
    locale: undefined,
    size: "md",
  },
);

const fieldId = useId(props.id);
const defaultLocale = useLocale();

const dateValue = computed<DateValue | null>({
  get() {
    if (!model.value) return null;
    try {
      return parseDate(model.value);
    } catch (error) {
      console.warn(`[UiDateField] Failed to parse date "${model.value}":`, error);
      return null;
    }
  },
  set(value) {
    model.value = value ? value.toString() : undefined;
  },
});
</script>

<template>
  <DateFieldRoot
    v-model="dateValue"
    v-slot="{ segments }"
    :id="fieldId"
    :disabled="disabled"
    :locale="locale ?? defaultLocale"
    :class="
      cn(
        'flex w-full items-center select-none focus-within:border-focus-ring',
        INPUT_BASE_CLASSES,
        size === 'sm'
          ? 'h-(--knovum-control-height-sm) px-2 text-caption'
          : 'h-(--knovum-control-height-md) px-2.5 text-label',
        props.class,
      )
    "
  >
    <template v-for="item in segments" :key="item.part">
      <DateFieldInput
        :part="item.part"
        class="rounded-grid-cell px-0.5 transition-colors outline-none focus:bg-focus-ring/20 focus:text-foreground data-[placeholder]:text-foreground-tertiary"
      >
        {{ item.value }}
      </DateFieldInput>
    </template>
  </DateFieldRoot>
</template>
