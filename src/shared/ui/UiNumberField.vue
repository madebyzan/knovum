<script setup lang="ts">
import { NumberFieldInput, NumberFieldRoot, useId, useLocale } from "reka-ui";
import { computed, type HTMLAttributes } from "vue";

import { cn } from "./cn";
import { INPUT_BASE_CLASSES } from "./ui-classes";

const model = defineModel<number | undefined>();

const props = withDefaults(
  defineProps<{
    id?: string | undefined;
    min?: number | undefined;
    max?: number | undefined;
    step?: number | undefined;
    disabled?: boolean | undefined;
    placeholder?: string | undefined;
    locale?: string | undefined;
    formatOptions?: Intl.NumberFormatOptions | undefined;
    size?: "sm" | "md" | undefined;
    class?: HTMLAttributes["class"];
  }>(),
  {
    disabled: false,
    placeholder: "",
    size: "md",
  },
);

const fieldId = useId(props.id);
const defaultLocale = useLocale();

const numberValue = computed<number | null>({
  get: () => model.value ?? null,
  set: (value) => {
    model.value = value ?? undefined;
  },
});

const rootProps = computed(() => ({
  id: fieldId,
  disabled: props.disabled,
  locale: props.locale ?? defaultLocale.value,
  ...(props.min !== undefined ? { min: props.min } : {}),
  ...(props.max !== undefined ? { max: props.max } : {}),
  ...(props.step !== undefined ? { step: props.step } : {}),
  ...(props.formatOptions ? { formatOptions: props.formatOptions } : {}),
}));
</script>

<template>
  <NumberFieldRoot v-bind="rootProps" v-model="numberValue" class="w-full">
    <NumberFieldInput
      :placeholder="placeholder"
      :class="
        cn(
          'w-full [appearance:textfield] [&::-webkit-inner-spin-button]:hidden [&::-webkit-outer-spin-button]:hidden',
          INPUT_BASE_CLASSES,
          size === 'sm'
            ? 'h-(--knovum-control-height-sm) px-2 text-caption'
            : 'h-(--knovum-control-height-md) px-2.5 text-label',
          props.class,
        )
      "
    />
  </NumberFieldRoot>
</template>
