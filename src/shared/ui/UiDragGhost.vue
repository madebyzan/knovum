<script setup lang="ts">
/** What is being dragged, under the pointer; clear only where it can be dropped. */
defineProps<{ x: number; y: number; droppable: boolean }>();
</script>

<template>
  <Teleport to="body">
    <div
      aria-hidden="true"
      class="pointer-events-none fixed z-100 flex max-w-64 items-center gap-1.5 rounded-md border border-stroke-default bg-surface-raised px-2 py-1 text-caption text-fg-primary shadow-popover transition-opacity duration-100"
      :class="droppable ? 'opacity-100' : 'opacity-50'"
      :style="{ left: `${x + 12}px`, top: `${y + 12}px` }"
    >
      <slot />
    </div>
  </Teleport>
</template>
