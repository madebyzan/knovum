<script setup lang="ts">
export interface ReaderOutlineAnchor {
  key: string | number;
  label: string;
}

defineProps<{ anchors: readonly ReaderOutlineAnchor[] }>();
const emit = defineEmits<{ select: [key: string | number] }>();
</script>

<template>
  <div class="flex min-w-0 items-center gap-1 overflow-x-auto">
    <button
      v-for="anchor in anchors"
      :key="anchor.key"
      type="button"
      class="max-w-44 shrink-0 truncate rounded-md border border-stroke-divider/70 bg-surface-raised px-2 py-0.5 text-caption-sm font-medium text-fg-secondary hover:bg-surface-control-hover hover:text-fg-primary"
      @click="emit('select', anchor.key)"
    >
      {{ anchor.label }}
    </button>
  </div>
</template>
