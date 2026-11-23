<script setup lang="ts">
import { Grip, Grid2x2, Square } from "@lucide/vue";
import { storeToRefs } from "pinia";
import { computed, shallowRef, type Component } from "vue";
import { useI18n } from "vue-i18n";

import { CANVAS_PATTERN_LABEL_KEY } from "../../../../foundation/preferences/canvas-options";
import { usePreferencesStore } from "../../../../foundation/preferences/preferences-store";
import type { CanvasBackgroundPattern } from "../../../../shared/contracts/generated/CanvasBackgroundPattern";
import { canvasBackgroundPatterns } from "../../../../shared/contracts/generated/contract-values";
import UiIconButton from "../../../../shared/ui/UiIconButton.vue";
import { useSpaceStore } from "../../session/space-store";

const PATTERN_ICON_BY_VALUE: Readonly<Record<CanvasBackgroundPattern, Component>> = {
  dots: Grip,
  grid: Grid2x2,
  none: Square,
};

/** The trigger lives in the overflow-clipped bottom toolbar, so the choices teleport to the
 *  viewport and unfold in a row along the toolbar to the right of the measured icon. */
const ORBIT_STEP_PX = 34;

const { t } = useI18n();
const store = useSpaceStore();
const { canvas } = storeToRefs(usePreferencesStore());
const open = shallowRef(false);
const triggerWrap = shallowRef<HTMLElement>();
const orbitCenter = shallowRef({ x: 0, y: 0 });

const effectiveSpacePattern = computed(() => store.backgroundPattern ?? canvas.value.backgroundPattern);

function toggle() {
  if (!open.value) {
    const rect = triggerWrap.value?.getBoundingClientRect();
    if (rect) orbitCenter.value = { x: rect.left + rect.width / 2, y: rect.top + rect.height / 2 };
  }
  open.value = !open.value;
}

function orbitStyle(index: number) {
  const x = open.value ? (index + 1) * ORBIT_STEP_PX : 0;
  const stagger = open.value ? index : canvasBackgroundPatterns.length - 1 - index;
  return {
    left: `${orbitCenter.value.x}px`,
    top: `${orbitCenter.value.y}px`,
    transform: `translate(-50%, -50%) translateX(${x}px) scale(${open.value ? 1 : 0})`,
    opacity: open.value ? "1" : "0",
    transitionDelay: `${stagger * 45}ms`,
  };
}

function pick(value: CanvasBackgroundPattern) {
  store.setBackgroundPattern(value);
  open.value = false;
}
</script>

<template>
  <div ref="triggerWrap" @keydown.esc="open = false">
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
      :aria-expanded="open"
      @click="toggle"
    >
      <component :is="PATTERN_ICON_BY_VALUE[effectiveSpacePattern]" class="size-3" :stroke-width="1.55" />
    </UiIconButton>
  </div>
  <Teleport to="body">
    <div
      v-if="open"
      data-pattern-dismiss
      class="fixed inset-0 z-40 cursor-default"
      @pointerdown="open = false"
    />
    <button
      v-for="(value, index) in canvasBackgroundPatterns"
      :key="value"
      type="button"
      data-pattern-orbit
      :data-pattern-value="value"
      :aria-label="t(CANVAS_PATTERN_LABEL_KEY[value])"
      :aria-pressed="value === effectiveSpacePattern"
      :inert="!open"
      :style="orbitStyle(index)"
      :class="value === effectiveSpacePattern ? 'text-fg-accent' : 'text-fg-secondary'"
      class="fixed z-50 flex size-7 cursor-default items-center justify-center rounded-full border border-stroke-default bg-surface-raised shadow-control transition-[transform,opacity] duration-300 ease-[cubic-bezier(0.34,1.56,0.64,1)] outline-none hover:border-stroke-strong hover:bg-surface-control-hover focus-visible:ring-2 focus-visible:ring-stroke-focus motion-reduce:transition-none"
      @keydown.esc="open = false"
      @click="pick(value)"
    >
      <component :is="PATTERN_ICON_BY_VALUE[value]" class="size-3.5" :stroke-width="1.6" />
    </button>
  </Teleport>
</template>
