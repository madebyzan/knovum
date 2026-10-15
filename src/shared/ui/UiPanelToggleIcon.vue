<script setup lang="ts">
/**
 * Sidebar toggle glyph: one geometry for both sidebars and both states, on Lucide's 24px panel grid so it
 * sits with the rest of the toolbar. Closed, the pane is an empty slot; open, it is filled. The fill grows
 * from the window edge on the panel clock, so the icon makes the motion the panel makes instead of swapping
 * glyphs - two stacked line glyphs cross-fading inside 18px paint both strokes at once, a visible flash.
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
    stroke-width="1.55"
    stroke-linecap="round"
    stroke-linejoin="round"
    aria-hidden="true"
  >
    <path
      class="ui-panel-toggle-icon-pane"
      :d="
        side === 'left'
          ? 'M9 3H5a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h4z'
          : 'M15 3h4a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2h-4z'
      "
      fill="currentColor"
      stroke="none"
    />
    <rect x="3" y="3" width="18" height="18" rx="2" />
    <path :d="side === 'left' ? 'M9 3v18' : 'M15 3v18'" />
  </svg>
</template>

<style scoped>
.ui-panel-toggle-icon-pane {
  opacity: 0;
  transform: scaleX(0);
  transform-box: fill-box;
  transform-origin: left center;
  transition:
    transform var(--knovum-panel-motion-duration) var(--knovum-panel-motion-easing),
    opacity var(--knovum-panel-motion-duration) var(--knovum-panel-motion-easing);
}

.ui-panel-toggle-icon[data-side="right"] .ui-panel-toggle-icon-pane {
  transform-origin: right center;
}

.ui-panel-toggle-icon[data-open] .ui-panel-toggle-icon-pane {
  opacity: 1;
  transform: none;
}
</style>
