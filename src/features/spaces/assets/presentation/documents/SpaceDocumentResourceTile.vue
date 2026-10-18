<script setup lang="ts">
import { ImageOff, Maximize2 } from "@lucide/vue";
import { computed, shallowRef, watch } from "vue";
import { useI18n } from "vue-i18n";

import { formatByteSize } from "../../format-byte-size";
import { spaceAssetSource } from "../../node-preview-source";
import type { DocumentResourceProjection } from "./document-resource-projection";

const props = defineProps<{
  item: DocumentResourceProjection;
  index: number;
  previewHeight: number;
  sourceRevision: string;
}>();
const emit = defineEmits<{ select: [] }>();
const { t } = useI18n();
const source = computed(() => spaceAssetSource(props.item.resource.path, props.sourceRevision));
const failed = shallowRef(false);
watch(source, () => {
  failed.value = false;
});
</script>

<template>
  <button
    type="button"
    :data-resource-id="item.resource.id"
    class="group relative block w-full cursor-pointer select-none overflow-hidden rounded-lg bg-surface-panel text-left ring-1 ring-stroke-divider ring-inset transition-colors duration-150 hover:bg-surface-control-hover hover:ring-stroke-default-hover focus-visible:outline-2 focus-visible:-outline-offset-2 focus-visible:outline-stroke-focus"
    :aria-label="`${t('spaces.processing.document.resources.open', { index: index + 1 })}: ${item.locationLabel}${failed ? ` · ${t('spaces.processing.imageUnavailable')}` : ''}`"
    @click="emit('select')"
  >
    <div
      data-resource-preview
      class="relative grid size-full place-items-center overflow-hidden bg-surface-muted"
      :style="{ height: `${previewHeight}px` }"
    >
      <div v-if="failed" class="flex flex-col items-center gap-1.5 px-2 text-center text-fg-tertiary">
        <ImageOff class="size-4" :stroke-width="1.5" />
        <span class="text-caption-sm font-medium">{{ t("spaces.processing.imageUnavailable") }}</span>
        <span class="max-w-full truncate text-caption-sm text-fg-tertiary/80">{{ item.locationLabel }}</span>
      </div>
      <img
        v-else
        :src="source"
        :width="item.resource.width"
        :height="item.resource.height"
        alt=""
        class="size-full object-contain"
        loading="lazy"
        decoding="async"
        draggable="false"
        @error="failed = true"
      />
      <span
        v-if="!failed"
        class="pointer-events-none absolute top-1.5 right-1.5 grid size-6 place-items-center rounded-md bg-black/55 text-fg-on-image shadow-control ring-1 ring-stroke-on-image/20 backdrop-blur-md opacity-0 transition-opacity duration-150 group-hover:opacity-100 group-focus-visible:opacity-100"
        aria-hidden="true"
      >
        <Maximize2 class="size-3" :stroke-width="1.8" />
      </span>
      <div
        v-if="!failed"
        class="pointer-events-none absolute inset-x-0 bottom-0 bg-gradient-to-t from-black/85 via-black/50 to-transparent px-2 pt-6 pb-1.5 opacity-0 transition-opacity duration-150 group-hover:opacity-100 group-focus-visible:opacity-100"
      >
        <strong
          :title="item.locationLabel"
          class="block truncate text-caption font-medium text-fg-on-image drop-shadow-sm"
        >
          {{ item.locationLabel }}
        </strong>
        <span class="mt-0.5 block truncate text-caption-sm text-fg-on-image/80 drop-shadow-sm">
          {{ item.resource.width }} × {{ item.resource.height }} · {{ formatByteSize(item.resource.byteSize) }}
        </span>
      </div>
    </div>
  </button>
</template>
