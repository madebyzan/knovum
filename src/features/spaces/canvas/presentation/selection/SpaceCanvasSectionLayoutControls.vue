<script setup lang="ts">
import { ChevronDown, Link, Link2Off } from "@lucide/vue";
import { ToolbarButton, ToolbarToggleGroup, ToolbarToggleItem } from "reka-ui";
import { computed, shallowRef } from "vue";
import { useI18n } from "vue-i18n";

import UiButton from "../../../../../shared/ui/UiButton.vue";
import UiDropdownMenu from "../../../../../shared/ui/UiDropdownMenu.vue";
import UiDropdownMenuItem from "../../../../../shared/ui/UiDropdownMenuItem.vue";
import UiIconButton from "../../../../../shared/ui/UiIconButton.vue";
import UiNumberField from "../../../../../shared/ui/UiNumberField.vue";
import UiPopover from "../../../../../shared/ui/UiPopover.vue";
import {
  CONTENT_FILL_PRESETS,
  type ContentFillPreset,
} from "../../../model/structure/schema/content-appearance";
import { SECTION_MINIMUM_SIZE } from "../../../model/structure/section-metrics";
import { CONTENT_FILL_COLORS } from "../canvas-content-colors";
import SpaceCanvasColorSwatches from "../SpaceCanvasColorSwatches.vue";
import {
  SECTION_SIZE_PRESETS,
  editedSectionSize,
  sectionSizePreset,
  type SectionSize,
} from "./section-frame-size";
import type { SelectedSection } from "./selection-product-state";

const props = defineProps<{ section: SelectedSection; disabled: boolean; compact: boolean }>();
const emit = defineEmits<{
  autoLayout: [autoLayout: boolean];
  fill: [fill: ContentFillPreset | undefined];
  resize: [size: SectionSize];
}>();
const { t } = useI18n();
const NO_FILL = "none";
const TOGGLE_ITEM_CLASSES =
  "h-[calc(var(--knovum-control-height-md)-4px)] rounded-[5px] px-2 text-caption text-fg-secondary outline-none transition-colors duration-100 hover:text-fg-primary focus-visible:ring-2 focus-visible:ring-stroke-focus disabled:opacity-40 data-[state=on]:bg-surface-raised data-[state=on]:text-fg-primary data-[state=on]:shadow-control";
const lockRatio = shallowRef(false);
const size = computed(() => ({ width: props.section.frame.width, height: props.section.frame.height }));
const preset = computed(() => sectionSizePreset(size.value));
const layoutMode = computed(() => (props.section.autoLayout ? "auto" : "manual"));
const fillOptions = computed(() => [
  { value: NO_FILL, color: "var(--color-surface-panel)", label: t("spaces.canvas.sections.fill.none") },
  ...CONTENT_FILL_PRESETS.map((value) => ({
    value,
    color: CONTENT_FILL_COLORS[value],
    label: t(`spaces.canvas.noteFills.${value}`),
  })),
]);

function selectLayout(mode: unknown) {
  // A single toggle group reports `undefined` when the active item is pressed again; the mode stays.
  if (mode === "auto" || mode === "manual") emit("autoLayout", mode === "auto");
}
function selectFill(value: string) {
  emit(
    "fill",
    CONTENT_FILL_PRESETS.find((fill) => fill === value),
  );
}
function edit(dimension: keyof SectionSize, value: number | undefined) {
  if (value !== undefined)
    emit("resize", editedSectionSize(size.value, { [dimension]: value }, lockRatio.value));
}
</script>

<template>
  <SpaceCanvasColorSwatches
    :label="t('spaces.canvas.sections.fill.label')"
    :options="fillOptions"
    :selected="section.appearance.fill ?? NO_FILL"
    @select="selectFill"
  />
  <ToolbarToggleGroup
    type="single"
    :model-value="layoutMode"
    :disabled="disabled"
    :aria-label="t('spaces.canvas.sections.layout.label')"
    class="flex h-(--knovum-control-height-md) items-center gap-px rounded-md bg-surface-control p-0.5"
    @update:model-value="selectLayout"
  >
    <ToolbarToggleItem
      value="auto"
      :class="TOGGLE_ITEM_CLASSES"
      :title="t('spaces.canvas.sections.layout.autoHint')"
    >
      {{ t("spaces.canvas.sections.layout.auto") }}
    </ToolbarToggleItem>
    <ToolbarToggleItem value="manual" :class="TOGGLE_ITEM_CLASSES">
      {{ t("spaces.canvas.sections.layout.manual") }}
    </ToolbarToggleItem>
  </ToolbarToggleGroup>
  <UiPopover side="top" align="center" :side-offset="6" class="w-60 p-2">
    <template #trigger>
      <ToolbarButton as-child>
        <UiButton
          variant="ghost"
          size="sm"
          class="tabular-nums shadow-none"
          :disabled="disabled"
          :aria-label="t('spaces.canvas.sections.size.label')"
        >
          <template v-if="compact">{{ preset ?? t("spaces.canvas.sections.size.custom") }}</template>
          <template v-else>{{ Math.round(size.width) }} × {{ Math.round(size.height) }}</template>
          <ChevronDown class="size-3 text-fg-tertiary" />
        </UiButton>
      </ToolbarButton>
    </template>
    <UiDropdownMenu side="bottom" align="start" class="min-w-44">
      <template #trigger>
        <UiButton variant="secondary" size="sm" class="w-full justify-between">
          {{ preset ?? t("spaces.canvas.sections.size.custom") }}
          <ChevronDown class="size-3 text-fg-tertiary" />
        </UiButton>
      </template>
      <UiDropdownMenuItem
        v-for="option in SECTION_SIZE_PRESETS"
        :key="option.id"
        @select="emit('resize', { width: option.width, height: option.height })"
      >
        <span class="flex-1">{{ option.id }}</span>
        <span class="text-caption text-fg-tertiary tabular-nums"
          >{{ option.width }} × {{ option.height }}</span
        >
      </UiDropdownMenuItem>
    </UiDropdownMenu>
    <div class="mt-2 flex items-center gap-1.5">
      <label class="flex min-w-0 flex-1 items-center gap-1 text-caption text-fg-tertiary">
        W
        <UiNumberField
          size="sm"
          :min="SECTION_MINIMUM_SIZE.width"
          :model-value="Math.round(size.width)"
          @update:model-value="edit('width', $event)"
        />
      </label>
      <UiIconButton
        :label="
          t(lockRatio ? 'spaces.canvas.sections.size.unlockRatio' : 'spaces.canvas.sections.size.lockRatio')
        "
        :active="lockRatio"
        tooltip
        size="sm"
        @click="lockRatio = !lockRatio"
      >
        <Link v-if="lockRatio" class="size-3.5" />
        <Link2Off v-else class="size-3.5" />
      </UiIconButton>
      <label class="flex min-w-0 flex-1 items-center gap-1 text-caption text-fg-tertiary">
        H
        <UiNumberField
          size="sm"
          :min="SECTION_MINIMUM_SIZE.height"
          :disabled="section.autoLayout"
          :model-value="Math.round(size.height)"
          @update:model-value="edit('height', $event)"
        />
      </label>
    </div>
    <p v-if="section.autoLayout" class="mt-1.5 text-caption text-fg-tertiary">
      {{ t("spaces.canvas.sections.size.autoHeight") }}
    </p>
  </UiPopover>
</template>
