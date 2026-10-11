<script setup lang="ts">
import {
  defaultRangeExtractor,
  observeElementRect,
  useVirtualizer,
  type Range,
  type Rect,
  type Virtualizer,
} from "@tanstack/vue-virtual";
import { computed, shallowRef, useTemplateRef, watch, type ComponentPublicInstance } from "vue";
import { useI18n } from "vue-i18n";

import type { DocumentResourceProjection } from "./document-resource-projection";
import SpaceDocumentResourceTile from "./SpaceDocumentResourceTile.vue";

const props = defineProps<{
  resources: readonly DocumentResourceProjection[];
  sourceRevision: string;
}>();
const emit = defineEmits<{ select: [resource: DocumentResourceProjection] }>();
const { t } = useI18n();
const scroller = useTemplateRef<HTMLDivElement>("scroller");
const width = shallowRef(0);
const RESOURCE_COLUMNS = 2;
const RESOURCE_GAP = 8;
const columnWidth = computed(() => (width.value - RESOURCE_GAP) / RESOURCE_COLUMNS);
const focusedResourceId = shallowRef<string>();
const virtualizer = useVirtualizer<HTMLDivElement, HTMLDivElement>(
  computed(() => {
    const resources = props.resources;
    const focusedIndex = resources.findIndex(({ resource }) => resource.id === focusedResourceId.value);
    return {
      count: resources.length,
      getScrollElement: () => scroller.value,
      getItemKey: (index: number) => resources[index]!.resource.id,
      estimateSize: (index: number) => previewHeight(resources[index]!) + 48,
      lanes: RESOURCE_COLUMNS,
      gap: RESOURCE_GAP,
      overscan: 4,
      observeElementRect: (
        instance: Virtualizer<HTMLDivElement, HTMLDivElement>,
        callback: (rect: Rect) => void,
      ) =>
        observeElementRect(instance, (rect) => {
          width.value = rect.width;
          callback(rect);
        }),
      rangeExtractor: (range: Range) => {
        const indexes = defaultRangeExtractor(range);
        if (focusedIndex < 0) return indexes;
        const focusNeighbors = [focusedIndex - 1, focusedIndex, focusedIndex + 1].filter(
          (index) => index >= 0 && index < resources.length,
        );
        return [...new Set([...indexes, ...focusNeighbors])].toSorted((left, right) => left - right);
      },
    };
  }),
);
const items = computed(() =>
  virtualizer.value.getVirtualItems().map((item) => ({ item, resource: props.resources[item.index]! })),
);
const totalSize = computed(() => virtualizer.value.getTotalSize());
const scrollerStyle = computed(() => ({ height: `${Math.min(totalSize.value, 384)}px` }));
function previewHeight({ resource }: DocumentResourceProjection) {
  return Math.min(288, Math.max(64, (columnWidth.value * resource.height) / resource.width));
}
function measureItem(element: Element | ComponentPublicInstance | null) {
  virtualizer.value.measureElement(element as HTMLDivElement | null);
}
watch(width, () => virtualizer.value.measure(), { flush: "post" });
</script>

<template>
  <div
    ref="scroller"
    role="list"
    :aria-label="t('spaces.processing.document.resources.title')"
    class="overflow-y-auto overscroll-contain"
    :style="scrollerStyle"
  >
    <div class="relative w-full" :style="{ height: `${totalSize}px` }">
      <div
        v-for="{ item, resource } in items"
        :key="String(item.key)"
        :ref="measureItem"
        :data-index="item.index"
        role="listitem"
        :aria-posinset="item.index + 1"
        :aria-setsize="resources.length"
        class="absolute top-0 left-0"
        :style="{
          width: `${columnWidth}px`,
          transform: `translate(${item.lane * (columnWidth + RESOURCE_GAP)}px, ${item.start}px)`,
        }"
        @focusin="focusedResourceId = resource.resource.id"
      >
        <SpaceDocumentResourceTile
          :item="resource"
          :index="item.index"
          :preview-height="previewHeight(resource)"
          :source-revision="sourceRevision"
          @select="emit('select', resource)"
        />
      </div>
    </div>
  </div>
</template>
