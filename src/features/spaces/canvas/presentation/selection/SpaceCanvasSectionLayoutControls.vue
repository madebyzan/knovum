<script setup lang="ts">
import { Check, Link, Link2Off } from "@lucide/vue";
import { ToolbarButton, ToolbarToggleGroup, ToolbarToggleItem } from "reka-ui";
import { computed, shallowRef } from "vue";
import { useI18n } from "vue-i18n";

import UiDropdownMenu from "../../../../../shared/ui/UiDropdownMenu.vue";
import UiDropdownMenuItem from "../../../../../shared/ui/UiDropdownMenuItem.vue";
import UiIconButton from "../../../../../shared/ui/UiIconButton.vue";
import UiSeparator from "../../../../../shared/ui/UiSeparator.vue";
import {
  CONTENT_FILL_PRESETS,
  type ContentFillPreset,
} from "../../../model/structure/schema/content-appearance";
import { SECTION_MINIMUM_SIZE } from "../../../model/structure/section-metrics";
import { CONTENT_FILL_COLORS } from "../canvas-content-colors";
import SpaceCanvasColorSwatches from "../SpaceCanvasColorSwatches.vue";
import SpaceCanvasMenuTrigger from "../SpaceCanvasMenuTrigger.vue";
import { NODE_ACTION_LABEL_CLASS, nodeActionLabelState } from "./node-action-label";
import {
  SECTION_SIZE_PRESETS,
  editedSectionSize,
  sectionSizePreset,
  type SectionSize,
  type SectionSizePreset,
} from "./section-frame-size";
import type { SelectedSection } from "./selection-product-state";
import SpaceCanvasAspectGlyph from "./SpaceCanvasAspectGlyph.vue";
import SpaceCanvasFrameDimension from "./SpaceCanvasFrameDimension.vue";

/** A Section's frame, edited the way a design tool edits one: fill, layout mode, then size. */
const props = defineProps<{ section: SelectedSection; disabled: boolean; compact: boolean }>();
const emit = defineEmits<{
  autoLayout: [autoLayout: boolean];
  fill: [fill: ContentFillPreset | undefined];
  resize: [size: SectionSize];
}>();
const { t } = useI18n();
const NO_FILL = "none";
const TOGGLE_ITEM_CLASSES =
  "inline-flex h-full items-center rounded-sm px-2.5 text-caption font-medium text-fg-secondary outline-none transition-colors duration-100 ease-standard hover:text-fg-primary focus-visible:ring-2 focus-visible:ring-stroke-focus disabled:opacity-40 data-[state=on]:bg-surface-raised data-[state=on]:text-fg-primary data-[state=on]:shadow-control motion-reduce:transition-none";
const lockRatio = shallowRef(false);
const size = computed(() => ({ width: props.section.frame.width, height: props.section.frame.height }));
const currentPreset = computed(() => sectionSizePreset(size.value));
const layoutMode = computed(() => (props.section.autoLayout ? "auto" : "manual"));
const fillOptions = computed(() => [
  { value: NO_FILL, color: "var(--color-surface-panel)", label: t("spaces.canvas.sections.fill.none") },
  ...CONTENT_FILL_PRESETS.map((value) => ({
    value,
    color: CONTENT_FILL_COLORS[value],
    label: t(`spaces.canvas.noteFills.${value}`),
  })),
]);
const ratioLabel = computed(() =>
  t(lockRatio.value ? "spaces.canvas.sections.size.unlockRatio" : "spaces.canvas.sections.size.lockRatio"),
);

function presetName(preset: SectionSizePreset) {
  return "named" in preset ? t(`spaces.canvas.sections.size.presets.${preset.id}`) : preset.id;
}
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
function edit(dimension: keyof SectionSize, value: number) {
  emit("resize", editedSectionSize(size.value, { [dimension]: value }, lockRatio.value));
}
</script>

<template>
  <SpaceCanvasColorSwatches
    :label="t('spaces.canvas.sections.fill.label')"
    :options="fillOptions"
    :selected="section.appearance.fill ?? NO_FILL"
    size="md"
    @select="selectFill"
  />
  <UiSeparator orientation="vertical" class="mx-1 h-4" />
  <span
    class="mr-1 text-caption text-fg-tertiary"
    :class="[NODE_ACTION_LABEL_CLASS, nodeActionLabelState(compact)]"
    aria-hidden="true"
  >
    {{ t("spaces.canvas.sections.layout.label") }}
  </span>
  <ToolbarToggleGroup
    type="single"
    :model-value="layoutMode"
    :disabled="disabled"
    :aria-label="t('spaces.canvas.sections.layout.label')"
    class="flex h-(--knovum-control-height-md) shrink-0 items-center gap-px rounded-md bg-surface-control p-0.5"
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
  <UiSeparator orientation="vertical" class="mx-1 h-4" />
  <UiDropdownMenu side="top" align="start" :side-offset="6" class="min-w-56">
    <template #trigger>
      <SpaceCanvasMenuTrigger
        :label="t('spaces.canvas.sections.size.label')"
        :disabled="disabled"
        class="gap-1.5 text-fg-primary"
      >
        <SpaceCanvasAspectGlyph v-bind="size" class="text-fg-secondary" />
        {{ currentPreset ? presetName(currentPreset) : t("spaces.canvas.sections.size.custom") }}
      </SpaceCanvasMenuTrigger>
    </template>
    <UiDropdownMenuItem
      v-for="option in SECTION_SIZE_PRESETS"
      :key="option.id"
      @select="emit('resize', { width: option.width, height: option.height })"
    >
      <SpaceCanvasAspectGlyph :width="option.width" :height="option.height" />
      <span class="flex-1">{{ presetName(option) }}</span>
      <span class="ml-4 text-caption text-fg-tertiary tabular-nums"
        >{{ option.width }} × {{ option.height }}</span
      >
      <Check class="size-3.5" :class="currentPreset?.id === option.id ? 'opacity-100' : 'opacity-0'" />
    </UiDropdownMenuItem>
  </UiDropdownMenu>
  <div class="ml-0.5 flex shrink-0 items-center gap-0.5">
    <SpaceCanvasFrameDimension
      axis="W"
      :label="t('spaces.canvas.sections.size.width')"
      :value="size.width"
      :min="SECTION_MINIMUM_SIZE.width"
      :disabled="disabled"
      @commit="edit('width', $event)"
    />
    <ToolbarButton as-child>
      <UiIconButton
        size="sm"
        tooltip
        :label="ratioLabel"
        :active="lockRatio"
        :disabled="disabled || section.autoLayout"
        @click="lockRatio = !lockRatio"
      >
        <component :is="lockRatio ? Link : Link2Off" class="size-3.5" :stroke-width="1.8" />
      </UiIconButton>
    </ToolbarButton>
    <SpaceCanvasFrameDimension
      axis="H"
      :label="t('spaces.canvas.sections.size.height')"
      :value="size.height"
      :min="SECTION_MINIMUM_SIZE.height"
      :disabled="disabled || section.autoLayout"
      :hint="section.autoLayout ? t('spaces.canvas.sections.size.autoHeight') : undefined"
      @commit="edit('height', $event)"
    />
  </div>
</template>
