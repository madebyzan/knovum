<script setup lang="ts">
import UiMarkdown from "../../../../shared/ui/UiMarkdown.vue";
import type { PdfTextPage } from "./pdf-editor-params";

defineProps<{ pages: readonly PdfTextPage[] }>();
</script>

<template>
  <div class="min-h-0 flex-1 overflow-y-auto bg-surface-raised px-7 py-6">
    <div class="mx-auto max-w-[68ch] space-y-6">
      <article
        v-for="item in pages"
        :key="item.page"
        :data-page-number="item.page"
        class="space-y-2.5 border-b border-stroke-divider/60 pb-5 last:border-b-0 last:pb-0"
      >
        <div data-reader-ignore-selection class="text-caption-sm font-semibold text-fg-tertiary select-none">
          PDF · p.{{ item.page }}
        </div>
        <UiMarkdown :content="item.text" class="leading-7 text-fg-primary" />
      </article>
    </div>
  </div>
</template>
