<script setup lang="ts">
import { ImagePlus, SplitSquareVertical } from "@lucide/vue";
import { useI18n } from "vue-i18n";

import type { ImageGroupLayout } from "../../../../shared/ui/image-group-layout";

defineProps<{
  layout: ImageGroupLayout;
  imageCount: number;
  visible: boolean;
}>();

const emit = defineEmits<{
  setLayout: [layout: ImageGroupLayout];
  addImages: [];
  splitGroup: [];
}>();

const { t } = useI18n();

const LAYOUT_OPTIONS: ReadonlyArray<{
  readonly id: ImageGroupLayout;
  readonly labelKey: string;
  readonly minImages?: number;
}> = [
  { id: "auto", labelKey: "spaces.richtext.image.group.auto" },
  { id: "gallery", labelKey: "spaces.richtext.image.group.gallery" },
  { id: "compare", labelKey: "spaces.richtext.image.group.compare", minImages: 2 },
  { id: "masonry", labelKey: "spaces.richtext.image.group.masonry" },
  { id: "cards", labelKey: "spaces.richtext.image.group.cards" },
  { id: "stack", labelKey: "spaces.richtext.image.group.stack" },
];
</script>

<template>
  <div
    contenteditable="false"
    data-image-group-interactive
    data-testid="image-group-bar"
    class="absolute inset-x-0 -top-9 z-20 flex items-center justify-between gap-2 pb-2 transition-opacity duration-150 ease-standard"
    :class="
      visible
        ? 'pointer-events-auto opacity-100'
        : 'pointer-events-none opacity-0 group-hover/img-group:pointer-events-auto group-hover/img-group:opacity-100'
    "
  >
    <div
      class="inline-flex items-center gap-0.5 rounded-lg border border-stroke-divider bg-surface-overlay/94 p-0.5 shadow-sm backdrop-blur-md"
    >
      <button
        v-for="option in LAYOUT_OPTIONS"
        :key="option.id"
        type="button"
        :data-testid="`image-group-layout-${option.id}`"
        :disabled="Boolean(option.minImages && imageCount < option.minImages)"
        class="h-6 rounded-md px-2 text-[11.5px] font-medium transition-colors disabled:pointer-events-none disabled:opacity-40"
        :class="
          layout === option.id
            ? 'bg-surface-selected text-fg-primary'
            : 'text-fg-secondary hover:bg-surface-control-hover hover:text-fg-primary'
        "
        @mousedown.prevent.stop
        @click.stop="emit('setLayout', option.id)"
      >
        {{ t(option.labelKey) }}
      </button>
    </div>

    <div
      class="inline-flex items-center gap-0.5 rounded-lg border border-stroke-divider bg-surface-overlay/94 p-0.5 shadow-sm backdrop-blur-md"
    >
      <button
        type="button"
        data-testid="image-group-add"
        :title="t('spaces.richtext.image.group.addImage')"
        :aria-label="t('spaces.richtext.image.group.addImage')"
        class="grid size-6 place-items-center rounded-md text-fg-secondary transition-colors hover:bg-surface-control-hover hover:text-fg-primary"
        @mousedown.prevent.stop
        @click.stop="emit('addImages')"
      >
        <ImagePlus class="size-3.5" :stroke-width="1.65" />
      </button>
      <button
        type="button"
        data-testid="image-group-split"
        :title="t('spaces.richtext.image.group.split')"
        :aria-label="t('spaces.richtext.image.group.split')"
        class="grid size-6 place-items-center rounded-md text-fg-secondary transition-colors hover:bg-surface-control-hover hover:text-fg-primary"
        @mousedown.prevent.stop
        @click.stop="emit('splitGroup')"
      >
        <SplitSquareVertical class="size-3.5" :stroke-width="1.65" />
      </button>
    </div>
  </div>
</template>
