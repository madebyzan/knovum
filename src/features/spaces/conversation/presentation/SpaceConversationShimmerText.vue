<script setup lang="ts">
defineProps<{
  /** Shows the three-stroke breathing mark in the leading 14px icon slot. */
  indicator?: boolean;
}>();
</script>

<template>
  <span v-if="indicator" class="inline-flex max-w-full min-w-0 items-center gap-1.5">
    <span
      class="inline-flex size-3.5 shrink-0 items-center justify-center gap-0.5 text-fg-secondary"
      aria-hidden="true"
    >
      <i
        v-for="bar in 3"
        :key="bar"
        class="conversation-work-mark w-0.5 rounded-full bg-current"
        :class="bar === 2 ? 'h-2.5' : 'h-2'"
      />
    </span>
    <span class="conversation-shimmer min-w-0 truncate"><slot /></span>
  </span>
  <span v-else class="conversation-shimmer"><slot /></span>
</template>

<style scoped>
/*
 * Waiting text reads as muted copy with one brighter band travelling through it. The band is the
 * primary foreground, never the accent: a coloured sweep turns a status line into decoration, and
 * the eye reads hue as meaning. A 250% gradient swept from 100% to 0% keeps the full text width
 * covered on every frame so no glyph is ever clipped by the background boundary.
 */
.conversation-shimmer {
  display: inline-block;
  max-width: 100%;
  color: var(--color-fg-secondary);
  background-image: linear-gradient(
    90deg,
    var(--color-fg-tertiary) 0%,
    var(--color-fg-secondary) 36%,
    var(--color-fg-primary) 50%,
    var(--color-fg-secondary) 64%,
    var(--color-fg-tertiary) 100%
  );
  background-size: 250% 100%;
  background-position: 100% 0;
  background-clip: text;
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  animation: conversation-shimmer 1.8s ease-in-out infinite;
}

.conversation-work-mark {
  animation: conversation-work-mark 1.5s ease-in-out infinite;
}

.conversation-work-mark:nth-child(2) {
  animation-delay: 160ms;
}

.conversation-work-mark:nth-child(3) {
  animation-delay: 320ms;
}

@keyframes conversation-shimmer {
  0% {
    background-position: 100% 0;
  }
  100% {
    background-position: 0% 0;
  }
}

@keyframes conversation-work-mark {
  0%,
  100% {
    opacity: 0.38;
    transform: scaleY(0.6);
  }
  50% {
    opacity: 1;
    transform: scaleY(1);
  }
}

@media (prefers-reduced-motion: reduce), (forced-colors: active) {
  .conversation-shimmer {
    animation: none;
    background-image: none;
    -webkit-text-fill-color: currentColor;
  }

  .conversation-work-mark {
    animation: none;
  }
}
</style>
