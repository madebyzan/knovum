<script setup lang="ts">
import { Grip, Grid2x2, Square } from "@lucide/vue";
import { useEventListener } from "@vueuse/core";
import { storeToRefs } from "pinia";
import { computed, nextTick, shallowRef, useTemplateRef, type Component } from "vue";
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

/**
 * The trigger lives in the overflow-clipped bottom toolbar, so the choices teleport to the
 * viewport and fan out along an equidistant quarter-circle arc (top, top-right, right).
 */
const ORBIT_RADIUS_PX = 38;
const ORBIT_ANGLES_RAD = [-Math.PI / 2, -Math.PI / 4, 0] as const;

const { t } = useI18n();
const store = useSpaceStore();
const { canvas } = storeToRefs(usePreferencesStore());
const open = shallowRef(false);
const triggerWrap = useTemplateRef<HTMLElement>("triggerWrap");
const orbitButtons = useTemplateRef<HTMLButtonElement[]>("orbitButtons");
const orbitCenter = shallowRef({ x: 0, y: 0 });

const effectiveSpacePattern = computed(() => store.backgroundPattern ?? canvas.value.backgroundPattern);

useEventListener(globalThis, "resize", () => {
  if (open.value) open.value = false;
});

function measureTrigger() {
  const rect = triggerWrap.value?.getBoundingClientRect();
  if (rect)
    orbitCenter.value = {
      x: Math.round(rect.left + rect.width / 2),
      y: Math.round(rect.top + rect.height / 2),
    };
}

function toggle() {
  if (!open.value) measureTrigger();
  open.value = !open.value;
}

function closeAndRestoreFocus() {
  open.value = false;
  void nextTick(() => triggerWrap.value?.querySelector("button")?.focus());
}

function orbitStyle(index: number) {
  const angle = ORBIT_ANGLES_RAD[index] ?? 0;
  const x = open.value ? Math.round(Math.cos(angle) * ORBIT_RADIUS_PX) : 0;
  const y = open.value ? Math.round(Math.sin(angle) * ORBIT_RADIUS_PX) : 0;
  const stagger = open.value ? index : canvasBackgroundPatterns.length - 1 - index;
  const delayMs = open.value ? stagger * 32 : stagger * 18;
  return {
    left: `${orbitCenter.value.x}px`,
    top: `${orbitCenter.value.y}px`,
    "--orbit-x": `${x}px`,
    "--orbit-y": `${y}px`,
    "--orbit-delay": `${delayMs}ms`,
  };
}

function handleOrbitKeydown(index: number, event: KeyboardEvent) {
  if (event.key === "Escape") {
    event.preventDefault();
    closeAndRestoreFocus();
    return;
  }
  const delta =
    event.key === "ArrowRight" || event.key === "ArrowDown"
      ? 1
      : event.key === "ArrowLeft" || event.key === "ArrowUp"
        ? -1
        : 0;
  if (!delta) return;
  event.preventDefault();
  const count = canvasBackgroundPatterns.length;
  orbitButtons.value?.[(index + delta + count) % count]?.focus();
}

function pick(value: CanvasBackgroundPattern) {
  store.setBackgroundPattern(value);
  open.value = false;
}
</script>

<template>
  <div ref="triggerWrap" @keydown.esc="closeAndRestoreFocus">
    <UiIconButton
      :label="t('spaces.canvas.background.title')"
      :tooltip="!open"
      size="sm"
      class="size-5 rounded-sm"
      :class="
        open
          ? 'bg-surface-control text-fg-primary'
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
      ref="orbitButtons"
      :key="value"
      type="button"
      data-pattern-orbit
      :data-pattern-value="value"
      :data-open="open"
      :aria-label="t(CANVAS_PATTERN_LABEL_KEY[value])"
      :title="t(CANVAS_PATTERN_LABEL_KEY[value])"
      :aria-pressed="value === effectiveSpacePattern"
      :inert="!open"
      :style="orbitStyle(index)"
      :class="
        value === effectiveSpacePattern
          ? 'border-stroke-accent/60 bg-surface-accent-subtle text-fg-accent'
          : 'border-stroke-strong bg-surface-overlay text-fg-secondary hover:border-stroke-default-hover hover:bg-surface-control-hover hover:text-fg-primary active:bg-surface-control-pressed'
      "
      class="pattern-orbit fixed z-50 flex size-6 cursor-default items-center justify-center rounded-full border outline-none focus-visible:ring-2 focus-visible:ring-stroke-focus"
      @keydown="handleOrbitKeydown(index, $event)"
      @click="pick(value)"
    >
      <component :is="PATTERN_ICON_BY_VALUE[value]" class="size-3" :stroke-width="1.55" />
    </button>
  </Teleport>
</template>

<style scoped>
.pattern-orbit {
  opacity: 0;
  pointer-events: none;
  transform: translate(-50%, -50%) translate3d(0, 0, 0) scale(0.55);
  transition-property: transform, opacity, background-color, border-color, color;
  transition-duration: 165ms, 115ms, 100ms, 100ms, 100ms;
  transition-timing-function:
    cubic-bezier(0.4, 0, 0.2, 1), ease-out, var(--ease-standard), var(--ease-standard), var(--ease-standard);
  transition-delay: var(--orbit-delay, 0ms), var(--orbit-delay, 0ms), 0ms, 0ms, 0ms;
}

.pattern-orbit[data-open="true"] {
  opacity: 1;
  pointer-events: auto;
  will-change: transform, opacity;
  transform: translate(-50%, -50%) translate3d(var(--orbit-x, 0px), var(--orbit-y, 0px), 0) scale(1);
  transition-duration: 240ms, 150ms, 100ms, 100ms, 100ms;
  transition-timing-function:
    cubic-bezier(0.22, 1.1, 0.36, 1), var(--ease-decelerate), var(--ease-standard), var(--ease-standard),
    var(--ease-standard);
}

@media (prefers-reduced-motion: reduce) {
  .pattern-orbit,
  .pattern-orbit[data-open="true"] {
    transition: none;
  }
}
</style>
