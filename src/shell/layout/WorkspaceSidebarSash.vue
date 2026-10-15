<script setup lang="ts">
import { useSidebarResize } from "./use-sidebar-resize";

const props = defineProps<{
  edge: "leading" | "trailing";
  controls: string;
  label: string;
  width: number;
  geometry: { minSize: number; maxSize: number };
}>();
const emit = defineEmits<{ resize: [value: number]; commit: [] }>();
const { begin, dragging, end, keydown, move } = useSidebarResize({
  edge: props.edge,
  geometry: props.geometry,
  width: () => props.width,
  onResize: (value) => emit("resize", value),
  onCommit: () => emit("commit"),
});
</script>

<template>
  <!--
    The divider takes no layout width: it sits exactly on the seam and reaches out for its hit area, so the
    gap between the sidebar and the work area stays the gap the layout asked for. A leading sidebar is
    flush with the work area, so its line sits on the seam; a trailing pane is separated by the work area's
    gutter, so its line centres in that gutter. It stacks above the sidebar slot it resizes.
  -->
  <div
    class="workspace-sash relative z-50 w-0 shrink-0 self-stretch outline-none"
    role="separator"
    aria-orientation="vertical"
    :aria-controls="controls"
    :aria-label="label"
    :aria-valuenow="width"
    :aria-valuemin="geometry.minSize"
    :aria-valuemax="geometry.maxSize"
    :data-dragging="dragging || undefined"
    tabindex="0"
    @keydown="keydown"
  >
    <span
      class="absolute inset-y-0 flex w-5 cursor-col-resize touch-none justify-center"
      :class="edge === 'trailing' ? '-left-3.5' : '-left-2.5'"
      @pointerdown="begin"
      @pointermove="move"
      @pointerup="end"
      @pointercancel="end"
      @lostpointercapture="end"
    >
      <span class="workspace-sash-line absolute inset-y-2 w-0.5 rounded-full" />
    </span>
  </div>
</template>

<style scoped>
.workspace-sash-line {
  background-color: transparent;
  transition: background-color 100ms var(--ease-standard);
}

.workspace-sash:hover .workspace-sash-line,
.workspace-sash:focus-visible .workspace-sash-line,
.workspace-sash[data-dragging] .workspace-sash-line {
  background-color: var(--color-surface-accent);
}
</style>
