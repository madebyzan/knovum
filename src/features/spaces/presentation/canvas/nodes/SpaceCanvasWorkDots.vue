<script setup lang="ts">
defineProps<{ placement: "field" | "inline" | "selection" }>();
</script>

<template>
  <span
    v-if="placement === 'inline'"
    class="inline-flex h-3.5 w-4.5 items-center justify-center gap-0.5"
    aria-hidden="true"
  >
    <i
      v-for="index in 3"
      :key="index"
      class="work-dot size-0.75 rounded-full bg-accent motion-reduce:animate-none"
    />
  </span>
  <span
    v-else
    class="work-dots-field pointer-events-none flex items-center justify-center overflow-hidden"
    :class="placement === 'field' ? 'size-24' : 'selection absolute inset-0'"
    aria-hidden="true"
  >
    <i class="work-dots-cluster block size-24 shrink-0 motion-reduce:animate-none" />
  </span>
</template>

<style scoped>
.work-dot {
  animation: work-dot-breath 2.4s ease-in-out infinite;
}

.work-dot:nth-child(2) {
  animation-delay: 160ms;
}
.work-dot:nth-child(3) {
  animation-delay: 320ms;
}

.work-dots-cluster {
  opacity: 0.38;
  background-image: radial-gradient(circle, var(--color-accent) 0.08rem, transparent 0.11rem);
  background-position: center;
  background-size: 1rem 1rem;
  mask-image: radial-gradient(circle, #000 18%, #0008 53%, transparent 73%);
  animation: work-field-breath 2.8s ease-in-out infinite alternate;
}

.selection .work-dots-cluster {
  opacity: 0.26;
}

@keyframes work-dot-breath {
  0%,
  100% {
    opacity: 0.38;
    transform: scale(0.7);
  }
  45% {
    opacity: 1;
    transform: scale(1.12);
  }
}

@keyframes work-field-breath {
  from {
    opacity: 0.28;
    transform: scale(0.94);
  }
  to {
    opacity: 0.58;
    transform: scale(1.04);
  }
}

@media (prefers-reduced-motion: reduce) {
  .selection .work-dots-cluster {
    opacity: 0.26;
  }
}
</style>
