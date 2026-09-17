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
        'grid size-4 shrink-0 place-items-center rounded-sm border border-border-strong bg-raised text-on-accent transition-colors outline-none hover:border-foreground focus-visible:border-focus-ring focus-visible:outline-none disabled:opacity-45 data-[state=checked]:border-accent data-[state=checked]:bg-accent',
        props.class,
      )
    "
  >
    <CheckboxIndicator><Check class="size-(--knovum-icon-size-xs)" :stroke-width="2.5" /></CheckboxIndicator>
  </CheckboxRoot>
</template>
