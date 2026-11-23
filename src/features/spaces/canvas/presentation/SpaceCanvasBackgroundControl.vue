<script setup lang="ts">
import { Grid2x2, Grip, Square } from "@lucide/vue";
import { storeToRefs } from "pinia";
import { RadioGroupItem, RadioGroupRoot } from "reka-ui";
import { computed, shallowRef, type Component } from "vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";

import { getErrorMessage } from "../../../../foundation/errors/error-format";
import { CANVAS_PATTERN_LABEL_KEY } from "../../../../foundation/preferences/canvas-options";
import { usePreferencesStore } from "../../../../foundation/preferences/preferences-store";
import type { CanvasBackgroundPattern } from "../../../../shared/contracts/generated/CanvasBackgroundPattern";
import { canvasBackgroundPatterns } from "../../../../shared/contracts/generated/contract-values";
import UiIconButton from "../../../../shared/ui/UiIconButton.vue";
import UiPopover from "../../../../shared/ui/UiPopover.vue";
import { useSpaceStore } from "../../session/space-store";

const PATTERN_ICON_BY_VALUE: Readonly<Record<CanvasBackgroundPattern, Component>> = {
  dots: Grip,
  grid: Grid2x2,
  none: Square,
};

const { t } = useI18n();
const store = useSpaceStore();
const preferences = usePreferencesStore();
const { canvas } = storeToRefs(preferences);
const open = shallowRef(false);

const effectiveSpacePattern = computed(() => store.backgroundPattern ?? canvas.value.backgroundPattern);

const patternOptions = computed(() =>
  canvasBackgroundPatterns.map((value) => ({
    value,
    label: t(CANVAS_PATTERN_LABEL_KEY[value]),
    icon: PATTERN_ICON_BY_VALUE[value],
  })),
);

function isPattern(value: unknown): value is CanvasBackgroundPattern {
  return typeof value === "string" && (canvasBackgroundPatterns as readonly string[]).includes(value);
}

function selectSpacePattern(value: unknown) {
  if (isPattern(value) && value !== store.backgroundPattern) {
    store.setBackgroundPattern(value);
  }
}

async function selectGlobalPattern(value: unknown) {
  if (!isPattern(value) || value === canvas.value.backgroundPattern) return;
  try {
    await preferences.setCanvasBackgroundPattern(value);
  } catch (error) {
    toast.error(`${t("settings.appearance.saveFailed")}: ${getErrorMessage(error)}`);
  }
}
</script>

<template>
  <UiPopover v-model:open="open" side="top" align="start" :side-offset="8" class="w-64 space-y-3 p-3">
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
      <div class="flex items-center justify-between gap-2">
        <span class="text-caption font-medium text-fg-secondary">
          {{ t("spaces.canvas.background.currentSpace") }}
        </span>
        <button
          v-if="store.backgroundPattern !== undefined"
          type="button"
          data-action="follow-global-pattern"
          class="rounded-sm px-1 text-caption-sm font-medium text-fg-accent transition-colors hover:bg-surface-control-hover focus-visible:ring-2 focus-visible:ring-stroke-focus focus-visible:outline-none"
          @click="store.setBackgroundPattern(undefined)"
        >
          {{ t("spaces.canvas.background.followGlobal") }}
        </button>
      </div>
      <RadioGroupRoot
        :model-value="effectiveSpacePattern"
        orientation="horizontal"
        :aria-label="t('spaces.canvas.background.currentSpace')"
        data-pattern-scope="space"
        class="grid grid-cols-3 rounded-md bg-surface-control p-0.5"
        @update:model-value="selectSpacePattern"
      >
        <RadioGroupItem
          v-for="option in patternOptions"
          :key="option.value"
          :value="option.value"
          class="flex h-7 cursor-default items-center justify-center gap-1.5 rounded-[5px] px-1.5 text-caption font-medium transition-[background-color,color,box-shadow] duration-100 ease-standard outline-none focus-visible:ring-2 focus-visible:ring-stroke-focus data-[state=checked]:bg-surface-raised data-[state=checked]:text-fg-primary data-[state=checked]:shadow-control data-[state=unchecked]:text-fg-secondary data-[state=unchecked]:hover:text-fg-primary"
          @click="selectSpacePattern(option.value)"
        >
          <component :is="option.icon" class="size-3.5 shrink-0" :stroke-width="1.6" />
          <span class="truncate">{{ option.label }}</span>
        </RadioGroupItem>
      </RadioGroupRoot>
    </div>
    <div class="space-y-1.5">
      <div class="text-caption font-medium text-fg-secondary">
        {{ t("spaces.canvas.background.globalDefault") }}
      </div>
      <RadioGroupRoot
        :model-value="canvas.backgroundPattern"
        orientation="horizontal"
        :aria-label="t('spaces.canvas.background.globalDefault')"
        data-pattern-scope="global"
        class="grid grid-cols-3 rounded-md bg-surface-control p-0.5"
        @update:model-value="selectGlobalPattern"
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
  </UiPopover>
</template>
