<script setup lang="ts">
import { FileText, Search } from "@lucide/vue";
import { computed } from "vue";
import { useI18n } from "vue-i18n";

import { truncateUtf16 } from "../../model/text";
import SpaceThumbnailPreview from "../preview/SpaceThumbnailPreview.vue";

const props = defineProps<{
  title: string;
  previewPath?: string;
  previewText?: string;
  fill?: boolean;
}>();
const { t } = useI18n();
const excerpt = computed(() =>
  props.previewText ? truncateUtf16(props.previewText.trim(), 12_000) : undefined,
);
</script>

<template>
  <div class="relative overflow-hidden bg-surface-canvas" :class="fill ? 'h-full' : 'h-72'">
    <SpaceThumbnailPreview v-if="previewPath" :path="previewPath" :title="title" />
    <div v-else-if="excerpt" class="relative h-full overflow-hidden bg-surface-muted px-4 py-3">
      <div class="mb-2 flex items-center gap-2 text-caption font-medium text-fg-secondary">
        <Search class="size-3.5 text-fg-tertiary" />
        {{ t("spaces.inspector.extractedDocumentText") }}
      </div>
      <pre class="font-sans text-caption leading-5 break-words whitespace-pre-wrap text-fg-secondary">{{
        excerpt
      }}</pre>
      <div
        class="pointer-events-none absolute inset-x-0 bottom-0 h-14 bg-linear-to-t from-surface-muted to-transparent"
      />
    </div>
    <div v-else class="grid h-full place-items-center px-8 text-center">
      <div>
        <FileText class="mx-auto size-7 text-fg-tertiary" :stroke-width="1.5" />
        <p class="mt-2 text-caption font-medium text-fg-secondary">
          {{ t("spaces.importExport.preview.instantUnavailable.title") }}
        </p>
        <p class="mt-1 text-caption-sm leading-relaxed text-fg-tertiary">
          {{ t("spaces.importExport.preview.instantUnavailable.description") }}
        </p>
      </div>
    </div>
  </div>
</template>
