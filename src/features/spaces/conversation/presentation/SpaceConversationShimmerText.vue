<template>
  <span class="conversation-shimmer"><slot /></span>
</template>

<style scoped>
/*
 * Waiting text reads as muted copy with one brighter band travelling through it. The band is the
 * primary foreground, never the accent: a coloured sweep turns a status line into decoration, and
 * the eye reads hue as meaning. Light text gets a brighter band, dark text a dimmer one, so the
 * motion stays legible in both appearances without a second palette.
 */
.conversation-shimmer {
  display: inline-block;
  max-width: 100%;
  color: var(--color-fg-secondary);
  background-image: linear-gradient(
    90deg,
    var(--color-fg-secondary) 0%,
    var(--color-fg-secondary) 40%,
    var(--color-fg-primary) 50%,
    var(--color-fg-secondary) 60%,
    var(--color-fg-secondary) 100%
  );
  background-repeat: no-repeat;
  background-size: 200% 100%;
  background-position: 150% 0;
  background-clip: text;
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  animation: conversation-shimmer 2.2s linear infinite;
}

@keyframes conversation-shimmer {
  to {
    background-position: -50% 0;
  }
}

@media (prefers-reduced-motion: reduce), (forced-colors: active) {
  .conversation-shimmer {
    animation: none;
    background-image: none;
    -webkit-text-fill-color: currentColor;
  }
}
</style>
