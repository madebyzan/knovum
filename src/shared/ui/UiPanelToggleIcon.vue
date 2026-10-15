<script setup lang="ts">
/**
 * Sidebar toggle glyph, one geometry for both sidebars and both states on Lucide's 24px grid. The pane is a
 * rounded block inset from the window frame, as a sidebar sits inside its window: closed it is an outline,
 * an empty place for the panel; open it is solid. The fill grows from the window edge on the panel clock,
 * so the icon makes the motion the panel makes, and the outline never moves so the state change is the
 * fill alone - two glyphs swapped or cross-faded in 18px read as a flash.
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
      <rect x="6" y="7.5" width="3.5" height="9" rx="0.625" stroke-width="1.25" />
      <rect
        class="ui-panel-toggle-icon-fill"
        x="5.375"
        y="6.875"
        width="4.75"
        height="10.25"
        rx="1.25"
        fill="currentColor"
        stroke="none"
      />
    </g>
  </svg>
</template>

<style scoped>
.ui-panel-toggle-icon-fill {
  transform: scaleX(0);
  transform-box: fill-box;
  transform-origin: left center;
  transition: transform var(--knovum-panel-motion-duration) var(--knovum-panel-motion-easing);
}

.ui-panel-toggle-icon[data-open] .ui-panel-toggle-icon-fill {
  transform: none;
}
</style>
