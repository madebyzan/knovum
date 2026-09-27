<script setup lang="ts">
import { Check, ChevronDown } from "@lucide/vue";
import {
  ComboboxAnchor,
  ComboboxContent,
  ComboboxEmpty,
  ComboboxInput,
  ComboboxItem,
  ComboboxItemIndicator,
  ComboboxPortal,
  ComboboxRoot,
  ComboboxTrigger,
  ComboboxViewport,
  useId,
} from "reka-ui";

import {
  INPUT_BASE_CLASSES,
  POPUP_ANIMATE_CLASSES,
  POPOVER_SURFACE_CLASSES,
} from "../../../../shared/ui/ui-classes";

const model = defineModel<string>({ required: true });
defineProps<{ families: readonly string[]; label: string; emptyLabel: string; disabled?: boolean }>();
const inputId = useId();
</script>

<template>
  <div class="min-w-0">
    <label :for="inputId" class="mb-1.5 block text-caption text-fg-secondary">{{ label }}</label>
    <ComboboxRoot v-model="model" :disabled="disabled" open-on-click>
      <ComboboxAnchor class="relative block">
        <ComboboxInput
          :id="inputId"
          :display-value="(value) => String(value ?? '')"
          :class="[
            INPUT_BASE_CLASSES,
            'h-(--knovum-control-height-sm) w-full min-w-0 pr-7 pl-2 text-caption',
          ]"
        />
        <ComboboxTrigger
          class="absolute inset-y-0 right-0 grid w-7 place-items-center rounded-r-md text-fg-tertiary hover:text-fg-primary focus-visible:outline-2 focus-visible:outline-stroke-focus"
          :aria-label="label"
        >
          <ChevronDown class="size-3" />
        </ComboboxTrigger>
      </ComboboxAnchor>
      <ComboboxPortal>
        <ComboboxContent
          position="popper"
          :side-offset="6"
          :collision-padding="8"
          :class="[
            POPUP_ANIMATE_CLASSES,
            POPOVER_SURFACE_CLASSES,
            'z-50 min-w-(--reka-combobox-trigger-width) p-1 duration-100 ease-standard',
          ]"
        >
          <ComboboxViewport class="max-h-60 overflow-y-auto">
            <ComboboxEmpty class="px-2 py-3 text-caption text-fg-secondary">{{ emptyLabel }}</ComboboxEmpty>
            <ComboboxItem
              v-for="family in families"
              :key="family"
              :value="family"
              class="relative flex min-h-(--knovum-list-item-height-sm) cursor-default items-center rounded-md py-1 pr-2 pl-7 text-label outline-none data-highlighted:bg-surface-control-hover"
            >
              <ComboboxItemIndicator class="absolute left-2"><Check class="size-3" /></ComboboxItemIndicator>
              <span class="min-w-0 truncate">{{ family }}</span>
            </ComboboxItem>
          </ComboboxViewport>
        </ComboboxContent>
      </ComboboxPortal>
    </ComboboxRoot>
  </div>
</template>
