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

const parsedDate = computed(() => {
  if (!model.value) return { value: null, invalid: false };
  try {
    return { value: parseDate(model.value), invalid: false };
  } catch {
    return { value: null, invalid: true };
  }
});
const dateValue = computed<DateValue | null>({
  get: () => parsedDate.value.value,
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
    :aria-invalid="parsedDate.invalid || undefined"
    :locale="locale ?? defaultLocale"
    :class="
      cn(
        'flex w-full items-center select-none focus-within:border-stroke-focus',
        INPUT_BASE_CLASSES,
        size === 'sm'
          ? 'h-(--knovum-control-height-sm) px-2 text-caption-md'
          : 'h-(--knovum-control-height-md) px-2.5 text-label-md',
        props.class,
      )
    "
  >
    <template v-for="item in segments" :key="item.part">
      <DateFieldInput
        :part="item.part"
        class="rounded-sm px-0.5 transition-colors outline-none focus:bg-surface-accent-subtle focus:text-fg-primary data-[placeholder]:text-fg-tertiary"
      >
        {{ item.value }}
      </DateFieldInput>
    </template>
  </DateFieldRoot>
</template>
