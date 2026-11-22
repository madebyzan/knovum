<script setup lang="ts">
import type { PdfBookmark } from "./pdf-outline";

defineProps<{
  bookmarks: readonly PdfBookmark[];
  currentPage: number;
}>();

const emit = defineEmits<{ select: [page: number] }>();
</script>

<template>
  <nav data-reader-ignore-selection class="px-2 py-2 select-none">
    <button
      v-for="(bookmark, idx) in bookmarks"
      :key="`${bookmark.page}:${idx}`"
      type="button"
      class="flex w-full items-baseline justify-between gap-2 rounded-md px-2 py-1 text-left text-caption transition-colors"
      :class="
        bookmark.page === currentPage
          ? 'bg-surface-accent-subtle/60 font-medium text-fg-primary'
          : 'text-fg-secondary hover:bg-surface-control-hover hover:text-fg-primary'
      "
      :style="{ paddingLeft: `${0.5 + bookmark.level * 0.75}rem` }"
      @click="emit('select', bookmark.page)"
    >
      <span class="truncate">{{ bookmark.title }}</span>
      <span class="shrink-0 text-caption-sm text-fg-tertiary tabular-nums">{{ bookmark.page }}</span>
    </button>
  </nav>
</template>
