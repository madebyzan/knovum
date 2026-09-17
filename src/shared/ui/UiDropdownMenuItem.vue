<script setup lang="ts">
import { DropdownMenuItem } from "reka-ui";
import type { HTMLAttributes } from "vue";

import { cn } from "./cn";
import { MENU_ITEM_BASE_CLASSES } from "./ui-classes";

const props = withDefaults(
  defineProps<{
    disabled?: boolean | undefined;
    destructive?: boolean | undefined;
    class?: HTMLAttributes["class"];
  }>(),
  {
    disabled: false,
    destructive: false,
  },
);

const emit = defineEmits<{
  select: [event: Event];
}>();
</script>

<template>
  <DropdownMenuItem
    :disabled="disabled"
    :class="
      cn(
        MENU_ITEM_BASE_CLASSES,
        'px-2',
        destructive ? 'text-danger data-highlighted:text-danger' : 'text-foreground',
        props.class,
      )
    "
    @select="emit('select', $event)"
  >
    <slot />
  </DropdownMenuItem>
</template>
