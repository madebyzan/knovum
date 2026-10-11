<script setup lang="ts">
import { ChevronRight } from "@lucide/vue";
import type { CSSProperties } from "vue";
import { useI18n } from "vue-i18n";

import UiScrollArea from "../../../../shared/ui/UiScrollArea.vue";
import { CROP_PRESET_GROUPS, type CropPreset } from "../../image-editing/crop-presets";

const props = defineProps<{ activeRatio?: number | undefined; fullCrop: boolean }>();
const emit = defineEmits<{ select: [ratio?: number] }>();
const { t } = useI18n();

function choose(preset: CropPreset) {
  emit("select", preset.ratio);
}

function selected(preset: CropPreset): boolean {
  return preset.ratio === undefined
    ? props.fullCrop && props.activeRatio === undefined
    : props.activeRatio !== undefined && Math.abs(props.activeRatio - preset.ratio) < 0.001;
}

function glyphStyle(ratio = 1): CSSProperties {
  return {
    width: `${16 * Math.min(1, ratio)}px`,
    height: `${16 / Math.max(1, ratio)}px`,
  };
}
</script>

<template>
  <div class="flex min-h-0 flex-1 flex-col">
    <p class="px-1 pb-2 text-caption text-fg-tertiary">
      {{ t("spaces.media.imageEditor.crop.presets") }}
    </p>
    <UiScrollArea class="min-h-0 flex-1">
      <details
        v-for="group in CROP_PRESET_GROUPS"
        :key="group.id"
        :open="group.id === 'general'"
        class="group pr-3"
      >
        <summary
          class="flex h-(--knovum-control-height-md) cursor-pointer list-none items-center gap-2 rounded-md px-2 text-label text-fg-primary hover:bg-surface-control-hover focus-visible:ring-2 focus-visible:ring-stroke-focus"
        >
          <ChevronRight class="size-3 shrink-0 text-fg-tertiary transition-transform group-open:rotate-90" />
          {{ group.id === "general" ? t("spaces.media.imageEditor.crop.general") : group.id }}
        </summary>
        <div class="pb-1 pl-4">
          <button
            v-for="preset in group.presets"
            :key="preset.id"
            type="button"
            class="flex h-(--knovum-control-height-md) w-full items-center gap-2 rounded-md px-2 text-left text-label text-fg-primary hover:bg-surface-control-hover focus-visible:ring-2 focus-visible:ring-stroke-focus"
            :class="selected(preset) ? 'bg-surface-control text-fg-primary' : ''"
            :aria-pressed="selected(preset)"
            @click="choose(preset)"
          >
            <span class="flex size-5 shrink-0 items-center justify-center rounded-md bg-surface-control">
              <span
                class="block rounded-sm border border-stroke-contrast-subtle"
                :style="glyphStyle(preset.ratio)"
              />
            </span>
            <span class="min-w-0 flex-1 truncate">{{
              preset.labelKey ? t(`spaces.media.imageEditor.${preset.labelKey}`) : preset.id
            }}</span>
            <span
              v-if="preset.referenceSize"
              class="shrink-0 text-caption text-fg-tertiary tabular-nums"
              :title="t('spaces.media.imageEditor.crop.referenceSize')"
            >
              {{ preset.referenceSize }}
            </span>
          </button>
        </div>
      </details>
    </UiScrollArea>
  </div>
</template>
