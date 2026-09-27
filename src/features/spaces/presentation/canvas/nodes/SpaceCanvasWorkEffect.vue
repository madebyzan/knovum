<script setup lang="ts">
defineProps<{ placement: "field" | "inline" | "selection" }>();
</script>

<template>
  <span
    v-if="placement === 'inline'"
    class="inline-flex h-3.5 w-4 items-center justify-center gap-0.5 text-current"
    aria-hidden="true"
  >
    <i v-for="index in 3" :key="index" class="work-mark h-2 w-0.5 rounded-full bg-current" />
  </span>
  <span
    v-else
    class="pointer-events-none absolute inset-0 overflow-hidden"
    :class="placement"
    aria-hidden="true"
  >
    <i class="work-light absolute inset-y-0 left-0 w-full" />
  </span>
</template>

<style scoped>
.work-light {
  background: radial-gradient(ellipse 55% 70% at center, var(--color-surface-skeleton), transparent 100%);
  animation: work-light-drift 2.6s ease-in-out infinite alternate;
}

.selection .work-light {
  opacity: 0.5;
}

.work-mark {
  animation: work-mark-breath 1.8s ease-in-out infinite;
}

.work-mark:nth-child(2) {
  animation-delay: 180ms;
}

.work-mark:nth-child(3) {
  animation-delay: 360ms;
}

@keyframes work-light-drift {
  from {
    transform: translateX(-20%) scale(1.2);
  }
  to {
    transform: translateX(20%) scale(1.2);
  }
}

@keyframes work-mark-breath {
  0%,
  100% {
    opacity: 0.45;
    transform: scaleY(0.65);
  }
  50% {
    opacity: 1;
    transform: scaleY(1);
  }
}

@media (prefers-reduced-motion: reduce) {
  .work-light,
  .work-mark {
    animation: none;
  }
}
</style>
