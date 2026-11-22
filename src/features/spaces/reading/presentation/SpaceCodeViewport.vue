<script setup lang="ts">
import { useVirtualizer } from "@tanstack/vue-virtual";
import { computed, nextTick, useTemplateRef, watch } from "vue";

const props = defineProps<{
  rawLines: readonly string[];
  highlightedLines?: readonly string[] | undefined;
  wrapLines: boolean;
  activeMatchLine?: number | undefined;
  highlightedRange?: { start: number; end: number } | undefined;
}>();

const scroller = useTemplateRef<HTMLDivElement>("scroller");
let measureEpoch = 0;

const virtualizer = useVirtualizer(
  computed(() => ({
    count: props.rawLines.length,
    getScrollElement: () => scroller.value,
    estimateSize: () => 22,
    overscan: 24,
  })),
);

const isTargetLine = (line: number) =>
  Boolean(
    props.highlightedRange && line >= props.highlightedRange.start && line <= props.highlightedRange.end,
  );

function measureLine(el: unknown) {
  if (!(el instanceof HTMLElement) || !props.wrapLines) return;
  const key = `${el.dataset.index}:${measureEpoch}`;
  if (el.dataset.measured === key) return;
  el.dataset.measured = key;
  virtualizer.value.measureElement(el);
}

watch(
  () => props.wrapLines,
  () => {
    measureEpoch += 1;
    void nextTick(() => virtualizer.value.measure());
  },
);

defineExpose({
  scrollToLine: (lineNumber: number) =>
    virtualizer.value.scrollToIndex(Math.max(0, lineNumber - 1), { align: "center" }),
});
</script>

<template>
  <div ref="scroller" class="min-h-0 flex-1 overflow-auto bg-surface-raised font-mono text-[13px]">
    <div
      class="relative py-2.5"
      :class="wrapLines ? 'w-full' : 'min-w-fit'"
      :style="{ height: `${virtualizer.getTotalSize() + 20}px` }"
    >
      <div
        v-for="vLine in virtualizer.getVirtualItems()"
        :key="vLine.index"
        :ref="measureLine"
        :data-index="vLine.index"
        :data-code-line="vLine.index + 1"
        class="group absolute top-0 left-0 flex w-full items-stretch leading-[1.65]"
        :class="
          activeMatchLine === vLine.index + 1
            ? 'bg-surface-accent-subtle/60'
            : isTargetLine(vLine.index + 1)
              ? 'bg-surface-accent-subtle/40'
              : 'hover:bg-surface-control/45'
        "
        :style="{ transform: `translateY(${vLine.start + 10}px)` }"
      >
        <span
          data-reader-ignore-selection
          class="w-12 shrink-0 border-r border-stroke-divider/70 bg-surface-panel/65 pr-2.5 text-right text-[11px] leading-[21.45px] text-fg-tertiary tabular-nums select-none"
          :class="
            (isTargetLine(vLine.index + 1) || activeMatchLine === vLine.index + 1) &&
            'border-r-surface-accent font-medium text-fg-accent'
          "
          >{{ vLine.index + 1 }}</span
        >
        <!-- eslint-disable-next-line vue/no-v-html -- escaped Shiki token spans -->
        <span
          v-if="highlightedLines?.[vLine.index] !== undefined"
          class="min-w-0 flex-1 px-3.5"
          :class="wrapLines ? 'break-all whitespace-pre-wrap' : 'whitespace-pre'"
          v-html="highlightedLines[vLine.index]"
        />
        <span
          v-else
          class="min-w-0 flex-1 px-3.5 text-fg-primary"
          :class="wrapLines ? 'break-all whitespace-pre-wrap' : 'whitespace-pre'"
          >{{ rawLines[vLine.index] || " " }}</span
        >
      </div>
    </div>
  </div>
</template>
