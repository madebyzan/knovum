<script setup lang="ts">
import { Check } from "@lucide/vue";
import { CheckboxIndicator, CheckboxRoot, useId } from "reka-ui";
import type { HTMLAttributes } from "vue";

import { cn } from "./cn";

const model = defineModel<boolean>({ required: true });

const props = withDefaults(
  defineProps<{
    id?: string | undefined;
    label: string;
    disabled?: boolean | undefined;
    class?: HTMLAttributes["class"];
  }>(),
  {
    id: undefined,
    disabled: false,
  },
);

const checkboxId = useId(props.id);
</script>

<template>
  <CheckboxRoot
    v-model="model"
    :id="checkboxId"
    :disabled="disabled"
    :aria-label="label"
    :class="
      cn(
        'grid size-4 shrink-0 place-items-center rounded-sm border border-stroke-strong bg-surface-raised text-fg-on-accent transition-colors outline-none hover:border-stroke-contrast focus-visible:border-stroke-focus focus-visible:outline-none disabled:opacity-45 data-[state=checked]:border-stroke-accent data-[state=checked]:bg-surface-accent',
        props.class,
      )
    "
  >
    <CheckboxIndicator><Check class="size-3" :stroke-width="2.5" /></CheckboxIndicator>
  </CheckboxRoot>
</template>
