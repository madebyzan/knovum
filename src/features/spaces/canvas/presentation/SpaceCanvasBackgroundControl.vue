<script setup lang="ts">
import { Grid2x2, Grip, Square } from "@lucide/vue";
import { storeToRefs } from "pinia";
import { RadioGroupItem, RadioGroupRoot } from "reka-ui";
import { computed, shallowRef, type Component } from "vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";

import { getErrorMessage } from "../../../../foundation/errors/error-format";
import {
  CANVAS_BACKDROP_LABEL_KEY,
  CANVAS_PATTERN_LABEL_KEY,
} from "../../../../foundation/preferences/canvas-options";
import { usePreferencesStore } from "../../../../foundation/preferences/preferences-store";
import type { CanvasBackdrop } from "../../../../shared/contracts/generated/CanvasBackdrop";
import type { CanvasBackgroundPattern } from "../../../../shared/contracts/generated/CanvasBackgroundPattern";
import {
  canvasBackdrops,
  canvasBackgroundPatterns,
} from "../../../../shared/contracts/generated/contract-values";
import UiIconButton from "../../../../shared/ui/UiIconButton.vue";
import UiPopover from "../../../../shared/ui/UiPopover.vue";
import { getCanvasBackdropAsset } from "../backdrops/backdrop-assets";

const PATTERN_ICON_BY_VALUE: Readonly<Record<CanvasBackgroundPattern, Component>> = {
  dots: Grip,
  grid: Grid2x2,
  none: Square,
};

const { t } = useI18n();
const preferences = usePreferencesStore();
const { canvas } = storeToRefs(preferences);
const open = shallowRef(false);

const patternOptions = computed(() =>
  canvasBackgroundPatterns.map((value) => ({
    value,
    label: t(CANVAS_PATTERN_LABEL_KEY[value]),
    icon: PATTERN_ICON_BY_VALUE[value],
  })),
);

const backdropOptions = computed(() =>
  canvasBackdrops.map((value) => ({
    value,
    label: t(CANVAS_BACKDROP_LABEL_KEY[value]),
    image: value === "none" ? undefined : getCanvasBackdropAsset(value),
  })),
);

async function applyPreference(operation: () => Promise<void>) {
  try {
    await operation();
  } catch (error) {
    toast.error(`${t("settings.appearance.saveFailed")}: ${getErrorMessage(error)}`);
  }
}

function selectPattern(value: unknown) {
  if (
    typeof value === "string" &&
    (canvasBackgroundPatterns as readonly string[]).includes(value) &&
    value !== canvas.value.backgroundPattern
  ) {
    void applyPreference(() => preferences.setCanvasBackgroundPattern(value as CanvasBackgroundPattern));
  }
}

function selectBackdrop(value: unknown) {
  if (
    typeof value === "string" &&
    (canvasBackdrops as readonly string[]).includes(value) &&
    value !== canvas.value.backdrop
  ) {
    void applyPreference(() => preferences.setCanvasBackdrop(value as CanvasBackdrop));
  }
}
</script>

<template>
  <UiPopover v-model:open="open" side="top" align="start" :side-offset="8" class="w-76 space-y-3.5 p-3">
    <template #trigger>
      <UiIconButton
        :label="t('spaces.canvas.background.title')"
        :tooltip="!open"
        size="sm"
        class="size-5 rounded-sm"
        :class="
          open
            ? 'bg-surface-control-hover text-fg-primary'
            : 'text-fg-secondary hover:bg-surface-control-hover hover:text-fg-primary'
        "
      >
        <Grid2x2 class="size-3" :stroke-width="1.55" />
      </UiIconButton>
    </template>
    <div class="space-y-1.5">
      <div class="text-caption font-medium text-fg-secondary">
        {{ t("settings.appearance.canvas.pattern.label") }}
      </div>
      <RadioGroupRoot
        :model-value="canvas.backgroundPattern"
        orientation="horizontal"
        :aria-label="t('settings.appearance.canvas.pattern.label')"
        class="grid grid-cols-3 rounded-md bg-surface-control p-0.5"
        @update:model-value="selectPattern"
      >
        <RadioGroupItem
          v-for="option in patternOptions"
          :key="option.value"
          :value="option.value"
          class="flex h-7 cursor-default items-center justify-center gap-1.5 rounded-[5px] px-1.5 text-caption font-medium transition-[background-color,color,box-shadow] duration-100 ease-standard outline-none focus-visible:ring-2 focus-visible:ring-stroke-focus data-[state=checked]:bg-surface-raised data-[state=checked]:text-fg-primary data-[state=checked]:shadow-control data-[state=unchecked]:text-fg-secondary data-[state=unchecked]:hover:text-fg-primary"
        >
          <component :is="option.icon" class="size-3.5 shrink-0" :stroke-width="1.6" />
          <span class="truncate">{{ option.label }}</span>
        </RadioGroupItem>
      </RadioGroupRoot>
    </div>
    <div class="space-y-2">
      <div class="text-caption font-medium text-fg-secondary">
        {{ t("settings.appearance.canvas.backdrop.label") }}
      </div>
      <RadioGroupRoot
        :model-value="canvas.backdrop"
        :aria-label="t('settings.appearance.canvas.backdrop.label')"
        class="grid grid-cols-3 gap-x-2.5 gap-y-2.5"
        @update:model-value="selectBackdrop"
      >
        <RadioGroupItem
          v-for="option in backdropOptions"
          :key="option.value"
          :value="option.value"
          class="group flex cursor-default flex-col items-center gap-1.5 rounded-md outline-none focus-visible:ring-2 focus-visible:ring-stroke-focus focus-visible:ring-offset-2 focus-visible:ring-offset-surface-overlay"
        >
          <span
            class="relative block aspect-16/10 w-full overflow-hidden rounded-md bg-surface-canvas transition-shadow duration-100 ease-standard group-data-[state=checked]:ring-2 group-data-[state=checked]:ring-stroke-accent group-data-[state=checked]:ring-offset-2 group-data-[state=checked]:ring-offset-surface-overlay group-data-[state=unchecked]:ring-1 group-data-[state=unchecked]:ring-stroke-strong group-data-[state=unchecked]:hover:ring-stroke-default-hover"
          >
            <img
              v-if="option.image"
              :src="option.image"
              alt=""
              decoding="async"
              draggable="false"
              class="size-full object-cover"
            />
            <svg
              v-else
              viewBox="0 0 80 50"
              preserveAspectRatio="none"
              class="size-full stroke-stroke-default"
              aria-hidden="true"
            >
              <line x1="0" y1="50" x2="80" y2="0" stroke-width="1" vector-effect="non-scaling-stroke" />
            </svg>
          </span>
          <span
            class="max-w-full truncate text-caption transition-colors duration-100 group-data-[state=checked]:text-fg-primary group-data-[state=unchecked]:text-fg-secondary group-data-[state=unchecked]:group-hover:text-fg-primary"
          >
            {{ option.label }}
          </span>
        </RadioGroupItem>
      </RadioGroupRoot>
    </div>
  </UiPopover>
</template>
