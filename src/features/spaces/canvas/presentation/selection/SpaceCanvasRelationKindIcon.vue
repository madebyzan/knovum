<script setup lang="ts">
/**
 * Minimal curved directed-relation glyph on Lucide's 24px optical grid, echoing the bezier
 * connectors drawn on Knovum's canvas.
 */
</script>

<template>
  <svg
    viewBox="0 0 24 24"
    fill="none"
    stroke="currentColor"
    stroke-width="1.65"
    stroke-linecap="round"
    stroke-linejoin="round"
    aria-hidden="true"
  >
    <circle cx="5.5" cy="17.5" r="2.25" />
    <path d="M7.75 17.5C12.75 17.5 11.25 6.5 18.25 6.5" />
    <path d="M15.75 4L18.5 6.5L15.75 9" />
  </svg>
</template>
