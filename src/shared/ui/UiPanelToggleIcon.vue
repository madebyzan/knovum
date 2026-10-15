<script setup lang="ts">
/**
 * Sidebar toggle glyph, one geometry for both sidebars and both states on Lucide's 24px grid. The pane is a
 * solid block inset from the window frame by the same padding on every side, with corners concentric to the
 * frame's, as a sidebar sits inside its window. Open, it is a full pane; closed, the same block narrows to a
 * rail against the window edge. The width itself animates - not a scale - so the corners stay round in every
 * frame, on the panel's duration and curve, so the icon makes the motion the panel makes. One shape that only
 * changes width cannot flash the way two glyphs swapped or cross-faded inside 18px do.
 */
defineProps<{ side: "left" | "right"; open: boolean }>();
</script>

<template>
  <svg
    class="ui-panel-toggle-icon"
    :data-side="side"
    :data-open="open || undefined"
    viewBox="0 0 24 24"
    fill="none"
    stroke="currentColor"
    stroke-width="1.5"
    stroke-linecap="round"
    stroke-linejoin="round"
    aria-hidden="true"
  >
    <rect x="2.75" y="4.25" width="18.5" height="15.5" rx="3.75" />
    <g :transform="side === 'right' ? 'matrix(-1 0 0 1 24 0)' : undefined">
      <rect
        class="ui-panel-toggle-icon-pane"
        x="5.375"
        y="6.875"
        height="10.25"
        rx="1.125"
        fill="currentColor"
        stroke="none"
      />
    </g>
  </svg>
</template>

<style scoped>
/*
 * x is the window-edge side (the group mirrors it for the right sidebar), so the width grows away from it.
 * The rail ends at x = 8, a whole pixel at 18px on 1x and 2x displays, so its edge is crisp at rest.
 */
.ui-panel-toggle-icon-pane {
  width: 2.625px;
  transition: width var(--knovum-panel-motion-duration) var(--knovum-panel-motion-easing);
}

.ui-panel-toggle-icon[data-open] .ui-panel-toggle-icon-pane {
  width: 4.75px;
}
</style>
