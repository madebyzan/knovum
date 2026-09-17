<script setup lang="ts">
import { SwitchRoot, SwitchThumb, useId } from "reka-ui";
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
        'relative h-5 w-9 shrink-0 rounded-full bg-control transition-colors duration-150 ease-standard outline-none focus-visible:outline-2 focus-visible:outline-offset-1 focus-visible:outline-focus-ring disabled:opacity-40 data-[state=checked]:bg-accent',
        props.class,
      )
    "
  >
    <SwitchThumb
      class="block size-4 translate-x-0.5 rounded-full bg-control-thumb shadow-control transition-transform duration-150 ease-standard data-[state=checked]:translate-x-4.5"
    />
  </SwitchRoot>
</template>
