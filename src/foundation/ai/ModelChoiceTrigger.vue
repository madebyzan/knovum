<script setup lang="ts">
import { ChevronRight } from "@lucide/vue";
import { DropdownMenuPortal, DropdownMenuSub, DropdownMenuSubContent, DropdownMenuSubTrigger } from "reka-ui";
import { shallowRef } from "vue";
import { useI18n } from "vue-i18n";

import { POPOVER_SURFACE_CLASSES, POPUP_ANIMATE_CLASSES } from "../../shared/ui/ui-classes";
import UiBadge from "../../shared/ui/UiBadge.vue";
import UiDropdownMenuItem from "../../shared/ui/UiDropdownMenuItem.vue";
import UiPopover from "../../shared/ui/UiPopover.vue";
import type { ModelChoiceOption } from "./model-choice";

withDefaults(
  defineProps<{
    options: readonly ModelChoiceOption[];
    menuItem?: boolean | undefined;
    disabled?: boolean | undefined;
    side?: "top" | "bottom" | "left" | "right" | undefined;
    align?: "start" | "center" | "end" | undefined;
  }>(),
  { side: "bottom", align: "start" },
);
const emit = defineEmits<{ select: [id: string] }>();
const { t } = useI18n();
const open = shallowRef(false);

function pick(id: string) {
  open.value = false;
  emit("select", id);
}
</script>

<template>
  <DropdownMenuSub v-if="menuItem && !disabled && options.length > 1">
    <DropdownMenuSubTrigger
      class="flex h-(--knovum-control-height-lg) w-full cursor-default items-center gap-2 rounded-md px-2 text-left text-label text-fg-primary outline-none data-highlighted:bg-surface-control-hover"
    >
      <slot name="menu-trigger" />
      <ChevronRight class="size-3 text-fg-tertiary" />
    </DropdownMenuSubTrigger>
    <DropdownMenuPortal>
      <DropdownMenuSubContent
        :side-offset="4"
        :align-offset="-4"
        :class="[
          POPOVER_SURFACE_CLASSES,
          POPUP_ANIMATE_CLASSES,
          'z-50 w-max max-w-80 min-w-52 p-1 shadow-popover outline-none',
        ]"
      >
        <UiDropdownMenuItem
          v-for="option in options"
          :key="option.id"
          class="justify-between gap-3"
          @select="emit('select', option.id)"
        >
          <span class="truncate">{{ option.name }}</span>
          <UiBadge v-if="option.preferred" size="sm" class="shrink-0">{{
            t("ai.modelChoice.default")
          }}</UiBadge>
        </UiDropdownMenuItem>
      </DropdownMenuSubContent>
    </DropdownMenuPortal>
  </DropdownMenuSub>
  <UiPopover
    v-else-if="!menuItem && !disabled && options.length > 1"
    v-model:open="open"
    :side="side"
    :align="align"
    class="w-max max-w-80 min-w-52 p-1"
  >
    <template #trigger>
      <slot name="trigger" :multiple="true" />
    </template>
    <button
      v-for="option in options"
      :key="option.id"
      type="button"
      class="flex h-(--knovum-control-height-lg) w-full items-center justify-between gap-3 rounded-md px-2 text-left text-label outline-none hover:bg-surface-control-hover focus-visible:ring-2 focus-visible:ring-stroke-focus"
      @click="pick(option.id)"
    >
      <span class="truncate font-medium text-fg-primary">{{ option.name }}</span>
      <UiBadge v-if="option.preferred" size="sm" class="shrink-0">{{ t("ai.modelChoice.default") }}</UiBadge>
    </button>
  </UiPopover>
  <slot v-else name="trigger" :multiple="false" />
</template>
