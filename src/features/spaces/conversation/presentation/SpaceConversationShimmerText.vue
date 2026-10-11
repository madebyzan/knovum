<template>
  <span class="conversation-shimmer"><slot /></span>
</template>

<style scoped>
.conversation-shimmer {
  display: inline-block;
  max-width: 100%;
  color: var(--color-fg-secondary);
  background: linear-gradient(
    105deg,
    var(--color-fg-secondary) 35%,
    var(--color-fg-accent) 48%,
    var(--color-fg-primary) 52%,
    var(--color-fg-secondary) 65%
  );
  background-size: 240% 100%;
  background-clip: text;
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  animation: conversation-shimmer 1.5s ease-in-out infinite;
}

@keyframes conversation-shimmer {
  from {
    background-position: 100% 0;
  }
  to {
    background-position: 0 0;
  }
}

@media (prefers-reduced-motion: reduce), (forced-colors: active) {
  .conversation-shimmer {
    animation: none;
    background: none;
    -webkit-text-fill-color: currentColor;
  }
}
</style>
