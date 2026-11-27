<script setup lang="ts">
import { SwitchRoot, SwitchThumb, useId } from "reka-ui";

import type { BinaryControlProps } from "./binary-control";
import { cn } from "./cn";

const model = defineModel<boolean>({ required: true });
const props = defineProps<BinaryControlProps>();

const switchId = useId(props.id);
</script>

<template>
  <SwitchRoot
    v-model="model"
    :id="switchId"
    :disabled="disabled"
    :aria-label="label"
    :class="
      cn(
        'relative h-5 w-9 shrink-0 rounded-full bg-surface-control transition-colors duration-150 ease-standard outline-none focus-visible:outline-2 focus-visible:outline-offset-1 focus-visible:outline-stroke-focus disabled:opacity-40 data-[state=checked]:bg-surface-accent',
        props.class,
      )
    "
  >
    <SwitchThumb
      class="block size-4 translate-x-0.5 rounded-full bg-white shadow-control transition-transform duration-150 ease-standard data-[state=checked]:translate-x-4.5"
    />
  </SwitchRoot>
</template>
