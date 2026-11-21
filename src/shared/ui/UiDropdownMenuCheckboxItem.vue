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
    trailingCheck?: boolean | undefined;
    class?: HTMLAttributes["class"];
  }>(),
  {
    disabled: false,
    trailingCheck: false,
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
    :class="
      cn(
        'relative',
        MENU_ITEM_BASE_CLASSES,
        trailingCheck ? 'px-2 text-fg-primary' : 'pr-2 pl-6 text-fg-primary',
        props.class,
      )
    "
    @select="emit('select', $event)"
  >
    <DropdownMenuItemIndicator
      v-if="!trailingCheck"
      class="absolute left-1.5 flex items-center justify-center"
    >
      <Check class="size-3 text-fg-accent" :stroke-width="2.2" />
    </DropdownMenuItemIndicator>
    <slot />
    <DropdownMenuItemIndicator
      v-if="trailingCheck"
      class="ml-auto flex shrink-0 items-center justify-center pl-2"
    >
      <Check class="size-3.5 text-fg-accent" :stroke-width="2.2" />
    </DropdownMenuItemIndicator>
  </DropdownMenuCheckboxItem>
</template>
