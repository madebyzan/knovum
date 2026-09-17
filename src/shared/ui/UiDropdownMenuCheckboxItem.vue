<script setup lang="ts">
import { Check } from "@lucide/vue";
import { DropdownMenuCheckboxItem, DropdownMenuItemIndicator } from "reka-ui";
import type { HTMLAttributes } from "vue";

import { cn } from "./cn";
import { MENU_ITEM_BASE_CLASSES } from "./ui-classes";

const checked = defineModel<boolean>("checked", { default: false });

const props = withDefaults(
  defineProps<{
    disabled?: boolean | undefined;
    class?: HTMLAttributes["class"];
  }>(),
  {
    disabled: false,
  },
);

const emit = defineEmits<{
  select: [event: Event];
}>();
</script>

<template>
  <DropdownMenuCheckboxItem
    v-model:checked="checked"
    :disabled="disabled"
    :class="cn('relative', MENU_ITEM_BASE_CLASSES, 'pr-2 pl-6 text-foreground', props.class)"
    @select="emit('select', $event)"
  >
    <DropdownMenuItemIndicator class="absolute left-1.5 flex items-center justify-center">
      <Check class="size-(--knovum-icon-size-xs) text-accent" :stroke-width="2.2" />
    </DropdownMenuItemIndicator>
    <slot />
  </DropdownMenuCheckboxItem>
</template>
