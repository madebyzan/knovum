<script setup lang="ts">
import { DropdownMenuContent, DropdownMenuPortal, DropdownMenuRoot, DropdownMenuTrigger } from "reka-ui";
import { computed, inject, watch, type Component, type HTMLAttributes } from "vue";

import { overlayScopeVisibleKey } from "../contracts/overlay-scope-visibility";
import { cn } from "./cn";
import { POPUP_ANIMATE_CLASSES, POPOVER_SURFACE_CLASSES } from "./ui-classes";
import UiDropdownMenuCheckboxItem from "./UiDropdownMenuCheckboxItem.vue";
import UiDropdownMenuItem from "./UiDropdownMenuItem.vue";
import UiKbd from "./UiKbd.vue";
import UiSeparator from "./UiSeparator.vue";

export interface UiDropdownMenuItemOption {
  id?: string | undefined;
  label?: string | undefined;
  icon?: Component | undefined;
  shortcut?: string | undefined;
  checked?: boolean | undefined;
  destructive?: boolean | undefined;
  disabled?: boolean | undefined;
  separator?: boolean | undefined;
  onSelect?: ((event: Event) => void) | undefined;
}

const open = defineModel<boolean>("open", { default: false });
const scopeVisible = inject(overlayScopeVisibleKey, undefined);
const scopeActive = computed(() => scopeVisible?.value !== false);
watch(
  scopeActive,
  (visible) => {
    if (!visible) open.value = false;
  },
  { flush: "sync" },
);

const props = withDefaults(
  defineProps<{
    items?: readonly UiDropdownMenuItemOption[] | undefined;
    side?: "top" | "right" | "bottom" | "left" | undefined;
    align?: "start" | "center" | "end" | undefined;
    sideOffset?: number | undefined;
    alignOffset?: number | undefined;
    modal?: boolean | undefined;
    class?: HTMLAttributes["class"];
  }>(),
  {
    items: undefined,
    side: "bottom",
    align: "end",
    sideOffset: 4,
    alignOffset: 0,
    modal: true,
  },
);
</script>

<template>
  <DropdownMenuRoot v-if="scopeActive" v-model:open="open" :modal="modal">
    <DropdownMenuTrigger v-if="$slots.trigger" as-child>
      <slot name="trigger" />
    </DropdownMenuTrigger>
    <DropdownMenuPortal>
      <DropdownMenuContent
        :side="side"
        :align="align"
        :side-offset="sideOffset"
        :align-offset="alignOffset"
        :collision-padding="8"
        avoid-collisions
        :class="
          cn(
            POPUP_ANIMATE_CLASSES,
            POPOVER_SURFACE_CLASSES,
            'z-50 min-w-36 p-1 duration-100 ease-standard outline-none select-none',
            props.class,
          )
        "
      >
        <slot name="header" />
        <template v-if="items">
          <template v-for="(item, i) in items" :key="item.id ?? item.label ?? i">
            <UiSeparator v-if="item.separator" class="my-1" />
            <UiDropdownMenuCheckboxItem
              v-else-if="item.checked !== undefined"
              :checked="item.checked"
              :disabled="item.disabled"
              @select="item.onSelect?.($event)"
            >
              <component :is="item.icon" v-if="item.icon" class="size-3 shrink-0" />
              <span class="min-w-0 flex-1 truncate">{{ item.label }}</span>
              <UiKbd v-if="item.shortcut" :shortcut="item.shortcut" class="ml-auto" />
            </UiDropdownMenuCheckboxItem>
            <UiDropdownMenuItem
              v-else
              :disabled="item.disabled"
              :destructive="item.destructive"
              @select="item.onSelect?.($event)"
            >
              <component :is="item.icon" v-if="item.icon" class="size-3 shrink-0" />
              <span class="min-w-0 flex-1 truncate">{{ item.label }}</span>
              <UiKbd v-if="item.shortcut" :shortcut="item.shortcut" class="ml-auto" />
            </UiDropdownMenuItem>
          </template>
        </template>
        <slot />
        <slot name="footer" />
      </DropdownMenuContent>
    </DropdownMenuPortal>
  </DropdownMenuRoot>
</template>
