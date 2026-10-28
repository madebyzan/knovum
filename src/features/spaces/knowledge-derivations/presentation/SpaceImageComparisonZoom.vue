<script setup lang="ts">
import { ZoomIn, ZoomOut } from "@lucide/vue";
import { useI18n } from "vue-i18n";

import UiTooltip from "../../../../shared/ui/UiTooltip.vue";

/** The comparison's zoom controls: step either way, or jump to the fitted or actual-size preset. */
defineProps<{
  scale: number;
  presets: { fitted: boolean; actualSize: boolean; zoomOut: boolean; zoomIn: boolean };
}>();
const emit = defineEmits<{ step: [direction: 1 | -1]; show: [preset: "fitted" | "actualSize"] }>();
const { t } = useI18n();
const CONTROL =
  "grid h-8 place-items-center rounded-lg font-medium transition-colors hover:bg-white/10 hover:text-white focus-visible:ring-2 focus-visible:ring-white/60 focus-visible:outline-none disabled:pointer-events-none disabled:opacity-40 aria-pressed:bg-white/15 aria-pressed:text-white";
</script>

<template>
  <div
    class="absolute bottom-4 left-1/2 flex h-10 -translate-x-1/2 items-center gap-0.5 rounded-xl bg-neutral-900/85 px-1 text-caption text-white/80 shadow-[0_8px_24px_rgb(0_0_0/0.4)] ring-1 ring-white/10 backdrop-blur-md"
  >
    <UiTooltip :text="t('spaces.media.imageComparison.zoomOut')" side="top">
      <button
        type="button"
        :class="[CONTROL, 'w-8']"
        :aria-label="t('spaces.media.imageComparison.zoomOut')"
        :disabled="!presets.zoomOut"
        @click="emit('step', -1)"
      >
        <ZoomOut class="size-4" :stroke-width="1.8" />
      </button>
    </UiTooltip>
    <span class="min-w-12 text-center font-medium text-white tabular-nums"
      >{{ Math.round(scale * 100) }}%</span
    >
    <UiTooltip :text="t('spaces.media.imageComparison.zoomIn')" side="top">
      <button
        type="button"
        :class="[CONTROL, 'w-8']"
        :aria-label="t('spaces.media.imageComparison.zoomIn')"
        :disabled="!presets.zoomIn"
        @click="emit('step', 1)"
      >
        <ZoomIn class="size-4" :stroke-width="1.8" />
      </button>
    </UiTooltip>
    <span class="mx-1 h-4 w-px bg-white/15" />
    <button
      type="button"
      :class="[CONTROL, 'px-2.5']"
      :aria-pressed="presets.fitted"
      @click="emit('show', 'fitted')"
    >
      {{ t("spaces.media.imageComparison.fit") }}
    </button>
    <UiTooltip :text="t('spaces.media.imageComparison.actualSize')" side="top">
      <button
        type="button"
        :class="[CONTROL, 'px-2.5 tabular-nums']"
        :aria-label="t('spaces.media.imageComparison.actualSize')"
        :aria-pressed="presets.actualSize"
        @click="emit('show', 'actualSize')"
      >
        100%
      </button>
    </UiTooltip>
  </div>
</template>
