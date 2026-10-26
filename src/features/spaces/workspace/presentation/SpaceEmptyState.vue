<script setup lang="ts">
import { Globe, Mic, Sparkles, StickyNote } from "@lucide/vue";
import { useI18n } from "vue-i18n";

import UiKbd from "../../../../shared/ui/UiKbd.vue";

const { t } = useI18n();
const WAVEFORM = [35, 70, 45, 88, 60, 35, 78, 50, 72, 40];
const PATHS = [
  "M52 0 C52 20 168 16 168 36",
  "M168 0 L168 36",
  "M284 0 C284 20 168 16 168 36",
] as const;
const SOURCE_ANCHORS = [52, 168, 284] as const;
const SHORTCUT_HINTS = [
  { key: "N", label: "spaces.editor.note.new" },
  { key: "T", label: "spaces.editor.text.new" },
  { key: "S", label: "spaces.canvas.tools.section" },
] as const;
const CARD_CLASS =
  "empty-rise flex items-center gap-2.5 rounded-lg border border-stroke-default bg-surface-raised px-2.5 py-2 shadow-panel";
const LINE_PRIMARY = "h-1.5 rounded-full bg-fg-primary/12";
const LINE_SECONDARY = "h-1.5 rounded-full bg-fg-tertiary/12";
const ICON_CLASS = "size-3.5 shrink-0 text-fg-tertiary";
</script>

<template>
  <div class="pointer-events-none absolute inset-0 grid place-items-center p-6 select-none">
    <div class="flex -translate-y-3 flex-col items-center text-center">
      <!-- Quiet monochromatic schematic: three sources converge into one cited note. -->
      <div class="flex w-84 flex-col items-center" aria-hidden="true">
        <div class="grid w-full grid-cols-3 gap-3">
          <div :class="CARD_CLASS" style="animation-delay: 0ms">
            <StickyNote :class="ICON_CLASS" :stroke-width="1.5" />
            <div class="min-w-0 flex-1 space-y-1.5">
              <div :class="[LINE_PRIMARY, 'w-4/5']" />
              <div :class="[LINE_SECONDARY, 'w-1/2']" />
            </div>
          </div>

          <div :class="CARD_CLASS" style="animation-delay: 50ms">
            <Globe :class="ICON_CLASS" :stroke-width="1.5" />
            <div class="min-w-0 flex-1 space-y-1.5">
              <div :class="[LINE_PRIMARY, 'w-full']" />
              <div :class="[LINE_SECONDARY, 'w-3/5']" />
            </div>
          </div>

          <div :class="CARD_CLASS" style="animation-delay: 100ms">
            <Mic :class="ICON_CLASS" :stroke-width="1.5" />
            <div class="flex h-3.5 flex-1 items-center justify-between gap-0.5">
              <span
                v-for="(height, index) in WAVEFORM"
                :key="index"
                class="w-0.5 rounded-full bg-fg-tertiary/30"
                :style="{ height: `${height}%` }"
              />
            </div>
          </div>
        </div>

        <svg class="h-9 w-full overflow-visible text-stroke-strong" viewBox="0 0 336 36" fill="none">
          <path
            v-for="path in PATHS"
            :key="path"
            :d="path"
            stroke="currentColor"
            stroke-width="1"
            stroke-dasharray="2.5 3"
          />
          <circle
            v-for="x in SOURCE_ANCHORS"
            :key="x"
            :cx="x"
            cy="0"
            r="1.75"
            fill="var(--color-surface-canvas)"
            stroke="currentColor"
            stroke-width="1"
          />
          <circle
            cx="168"
            cy="36"
            r="1.75"
            fill="var(--color-surface-canvas)"
            stroke="currentColor"
            stroke-width="1"
          />
        </svg>

        <div
          class="empty-rise w-48 rounded-lg border border-stroke-default bg-surface-raised p-2.5 text-left shadow-panel"
          style="animation-delay: 150ms"
        >
          <div class="flex items-center justify-between gap-2">
            <div class="flex items-center gap-1.5">
              <Sparkles class="size-3.5 shrink-0 text-fg-secondary" :stroke-width="1.5" />
              <div :class="[LINE_PRIMARY, 'w-16']" />
            </div>
            <div class="flex items-center gap-1">
              <span
                v-for="label in ['1', '2', '3']"
                :key="label"
                class="grid size-3.5 place-items-center rounded-xs bg-surface-muted text-[9px] leading-none font-medium text-fg-tertiary"
                >{{ label }}</span
              >
            </div>
          </div>
          <div class="mt-2 space-y-1.5">
            <div :class="[LINE_PRIMARY, 'w-full']" />
            <div :class="[LINE_SECONDARY, 'w-3/4']" />
          </div>
        </div>
      </div>

      <h2 class="mt-6 text-title-sm font-medium tracking-(--tracking-title) text-fg-primary">
        {{ t("spaces.space.empty.title") }}
      </h2>
      <p class="mt-1.5 max-w-md text-caption text-fg-tertiary">
        {{ t("spaces.space.empty.description") }}
      </p>

      <div class="mt-3.5 flex items-center justify-center gap-4 text-caption text-fg-tertiary">
        <span
          v-for="item in SHORTCUT_HINTS"
          :key="item.key"
          class="inline-flex items-center gap-1.5"
        >
          <UiKbd :shortcut="item.key" variant="badge" />
          <span>{{ t(item.label) }}</span>
        </span>
      </div>
    </div>
  </div>
</template>

<style scoped>
.empty-rise {
  animation: empty-rise 0.35s var(--ease-standard) backwards;
}
@media (prefers-reduced-motion: reduce) {
  .empty-rise {
    animation: none;
  }
}
@keyframes empty-rise {
  from {
    opacity: 0;
    transform: translateY(4px);
  }
}
</style>
