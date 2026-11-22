<script setup lang="ts">
import { Bookmark, LayoutGrid, ListTree } from "@lucide/vue";
import { shallowRef, watch } from "vue";
import { useI18n } from "vue-i18n";

import type { PdfReaderSession } from "./pdf-editor-params";
import type { PdfBookmark } from "./pdf-outline";
import SpacePdfHighlights from "./SpacePdfHighlights.vue";
import SpacePdfOutlineDrawer from "./SpacePdfOutlineDrawer.vue";
import SpacePdfThumbnails from "./SpacePdfThumbnails.vue";
import type { PdfProvenanceLink } from "./use-pdf-source-links";

const props = defineProps<{
  session?: PdfReaderSession | undefined;
  pageCount: number;
  currentPage: number;
  outline: readonly PdfBookmark[];
  links: readonly PdfProvenanceLink[];
  highlightedPages: ReadonlySet<number>;
}>();

const emit = defineEmits<{
  goToPage: [page: number];
  jumpLink: [link: PdfProvenanceLink];
  revealCanvas: [targetNodeId: string];
}>();

const { t } = useI18n();
const activeTab = shallowRef<"thumbnails" | "outline" | "highlights">("thumbnails");

watch(
  () => props.outline.length,
  (count) => {
    if (count === 0 && activeTab.value === "outline") activeTab.value = "thumbnails";
  },
);
</script>

<template>
  <aside
    data-reader-ignore-selection
    class="flex w-52 shrink-0 flex-col border-r border-stroke-divider bg-surface-panel/90 select-none"
  >
    <div class="grid shrink-0 grid-cols-3 gap-0.5 border-b border-stroke-divider p-1">
      <button
        type="button"
        :title="t('spaces.inspector.reader.thumbnails')"
        class="inline-flex h-7 items-center justify-center gap-1 rounded-sm text-caption-sm transition-colors"
        :class="
          activeTab === 'thumbnails'
            ? 'bg-surface-raised font-medium text-fg-primary shadow-2xs'
            : 'text-fg-secondary hover:bg-surface-control-hover'
        "
        @click="activeTab = 'thumbnails'"
      >
        <LayoutGrid class="size-3.5" />
      </button>
      <button
        type="button"
        :title="t('spaces.inspector.reader.outline')"
        :disabled="!outline.length"
        class="disabled:opacity-disabled inline-flex h-7 items-center justify-center gap-1 rounded-sm text-caption-sm transition-colors"
        :class="
          activeTab === 'outline'
            ? 'bg-surface-raised font-medium text-fg-primary shadow-2xs'
            : 'text-fg-secondary hover:bg-surface-control-hover'
        "
        @click="activeTab = 'outline'"
      >
        <ListTree class="size-3.5" />
      </button>
      <button
        type="button"
        :title="t('spaces.inspector.reader.highlights')"
        class="inline-flex h-7 items-center justify-center gap-1 rounded-sm text-caption-sm transition-colors"
        :class="
          activeTab === 'highlights'
            ? 'bg-surface-raised font-medium text-fg-primary shadow-2xs'
            : 'text-fg-secondary hover:bg-surface-control-hover'
        "
        @click="activeTab = 'highlights'"
      >
        <Bookmark class="size-3.5" />
        <span v-if="links.length" class="text-[10px] tabular-nums">{{ links.length }}</span>
      </button>
    </div>

    <div class="min-h-0 flex-1 overflow-y-auto">
      <SpacePdfThumbnails
        v-if="activeTab === 'thumbnails'"
        :session="session"
        :page-count="pageCount"
        :current-page="currentPage"
        :highlighted-pages="highlightedPages"
        @select="emit('goToPage', $event)"
      />
      <SpacePdfOutlineDrawer
        v-else-if="activeTab === 'outline'"
        :bookmarks="outline"
        :current-page="currentPage"
        @select="emit('goToPage', $event)"
      />
      <SpacePdfHighlights
        v-else
        :links="links"
        :current-page="currentPage"
        @jump-page="emit('jumpLink', $event)"
        @reveal-canvas="emit('revealCanvas', $event)"
      />
    </div>
  </aside>
</template>
