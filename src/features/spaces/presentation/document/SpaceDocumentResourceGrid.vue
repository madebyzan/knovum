<script setup lang="ts">
import { useVirtualizer } from "@tanstack/vue-virtual";
import { chunk } from "es-toolkit";
import { computed, useTemplateRef, type ComponentPublicInstance } from "vue";
import { useI18n } from "vue-i18n";

import { formatByteSize } from "../../assets/format-byte-size";
import { spaceAssetSource } from "../../assets/node-preview-source";
import type { DocumentResourceProjection } from "./document-resource-projection";

const props = defineProps<{
  resources: readonly DocumentResourceProjection[];
  sourceRevision: string;
}>();
const emit = defineEmits<{ select: [resource: DocumentResourceProjection] }>();
const { t } = useI18n();
const scroller = useTemplateRef<HTMLDivElement>("scroller");
const RESOURCE_COLUMNS = 2;
const resourceRows = computed(() => chunk(props.resources, RESOURCE_COLUMNS));
const virtualizer = useVirtualizer(
  computed(() => ({
    count: resourceRows.value.length,
    getScrollElement: () => scroller.value,
    getItemKey: (index: number) => resourceRows.value[index]![0]!.resource.id,
    estimateSize: () => 148,
    overscan: 3,
  })),
);
const rows = computed(() =>
  virtualizer.value.getVirtualItems().map((row) => ({ row, resources: resourceRows.value[row.index]! })),
);
const totalSize = computed(() => virtualizer.value.getTotalSize());
const scrollerStyle = computed(() => ({ height: `${Math.min(totalSize.value, 384)}px` }));
function measureRow(element: Element | ComponentPublicInstance | null) {
  virtualizer.value.measureElement(element as Element | null);
}
</script>

<template>
  <div ref="scroller" class="overflow-y-auto" :style="scrollerStyle">
    <div class="relative w-full" :style="{ height: `${totalSize}px` }">
      <div
        v-for="{ row, resources: rowResources } in rows"
        :key="String(row.key)"
        :ref="measureRow"
        :data-index="row.index"
        class="absolute top-0 left-0 grid w-full grid-cols-2 gap-2 pb-2"
        :style="{ transform: `translateY(${row.start}px)` }"
      >
        <button
          v-for="(item, column) in rowResources"
          :key="item.resource.id"
          type="button"
          class="overflow-hidden rounded-md border border-stroke-default bg-surface-muted text-left outline-none hover:border-stroke-default-hover focus-visible:ring-2 focus-visible:ring-stroke-accent"
          :aria-label="
            t('spaces.processing.document.resources.open', {
              index: row.index * RESOURCE_COLUMNS + column + 1,
            })
          "
          @click="emit('select', item)"
        >
          <div class="aspect-4/3 overflow-hidden bg-surface-canvas">
            <img
              :src="spaceAssetSource(item.resource.path, sourceRevision)"
              alt=""
              class="size-full object-contain"
              loading="lazy"
              decoding="async"
              draggable="false"
            />
          </div>
          <div class="border-t border-stroke-divider px-2 py-1.5">
            <strong class="block truncate text-caption font-medium text-fg-primary">
              {{ item.locationLabel }}
            </strong>
            <span class="block truncate text-caption-sm text-fg-tertiary">
              {{ item.resource.width }} × {{ item.resource.height }} ·
              {{ formatByteSize(item.resource.byteSize) }}
            </span>
          </div>
        </button>
      </div>
    </div>
  </div>
</template>
