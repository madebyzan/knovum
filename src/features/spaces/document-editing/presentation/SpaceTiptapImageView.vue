<script setup lang="ts">
import { convertFileSrc } from "@tauri-apps/api/core";
import { NodeViewWrapper, type NodeViewProps } from "@tiptap/vue-3";
import { computed, nextTick, shallowRef, useTemplateRef, watch } from "vue";
import { useI18n } from "vue-i18n";

import { isVideoMediaUrl } from "../../../../shared/ui/markdown-media";
import SpaceTiptapImageLightbox from "./SpaceTiptapImageLightbox.vue";
import SpaceTiptapImageToolbar from "./SpaceTiptapImageToolbar.vue";
import { IMAGE_RESIZE_SIDES, useSpaceTiptapImageResize } from "./use-space-tiptap-image-resize";

const props = defineProps<NodeViewProps>();
const { t } = useI18n();

const frameRef = useTemplateRef<HTMLElement>("frameRef");
const badgeRef = useTemplateRef<HTMLElement>("badgeRef");
const imgRef = useTemplateRef<HTMLImageElement>("imgRef");
const captionRef = useTemplateRef<HTMLTextAreaElement>("captionRef");
const editingCaption = shallowRef(false);
const lightboxOpen = shallowRef(false);

function syncCaptionHeight() {
  const el = captionRef.value;
  if (!el) return;
  el.style.height = "auto";
  if (el.scrollHeight > 0) el.style.height = `${el.scrollHeight}px`;
}

const { isEditable, renderedWidth, activePreset, setPresetFraction, startResize } = useSpaceTiptapImageResize(
  {
    frameRef,
    badgeRef,
    imgRef,
    getEditor: () => props.editor,
    getWidthAttr: () => props.node.attrs.width,
    updateSize: (width) => props.updateAttributes({ width, height: null }),
    onResizeEnd: () => void nextTick(syncCaptionHeight),
  },
);

const resolvedSrc = computed(() => {
  const assetPath = props.node.attrs.assetPath;
  if (typeof assetPath === "string" && assetPath) return convertFileSrc(assetPath);
  return typeof props.node.attrs.src === "string" ? props.node.attrs.src : "";
});
const isVideo = computed(() => isVideoMediaUrl(resolvedSrc.value));
const altText = computed(() => (typeof props.node.attrs.alt === "string" ? props.node.attrs.alt : ""));
const hasCaption = computed(() => altText.value.trim().length > 0);
const showCaption = computed(
  () => (isEditable.value && (props.selected || editingCaption.value)) || hasCaption.value,
);

watch([showCaption, altText, renderedWidth], () => void nextTick(syncCaptionHeight), { immediate: true });

function updateCaption(event: Event) {
  props.updateAttributes({ alt: (event.target as HTMLTextAreaElement).value || null });
  syncCaptionHeight();
}

function onCaptionKeydown(event: KeyboardEvent) {
  event.stopPropagation();
  if ((event.key === "Enter" && !event.shiftKey) || event.key === "Escape") {
    event.preventDefault();
    (event.target as HTMLTextAreaElement).blur();
  }
}
</script>

<template>
  <NodeViewWrapper as="figure" class="space-image-node group relative my-7 flex flex-col items-center">
    <div ref="frameRef" class="space-image-frame relative max-w-full">
      <div
        class="space-image-ring relative overflow-hidden rounded-lg transition-shadow duration-150 ease-standard select-none"
        :class="
          selected ? 'ring-[1.5px] ring-fg-accent/85' : 'ring-1 ring-black/8 ring-inset dark:ring-white/10'
        "
      >
        <video
          v-if="isVideo"
          :src="resolvedSrc"
          controls
          playsinline
          preload="metadata"
          class="block h-auto w-full bg-black"
        />
        <img
          v-else
          ref="imgRef"
          :src="resolvedSrc"
          :alt="altText"
          draggable="false"
          class="block h-auto w-full"
          @dblclick.stop="lightboxOpen = true"
        />
        <SpaceTiptapImageToolbar
          class="space-image-toolbar"
          :visible="selected"
          :is-editable="isEditable"
          :active-preset="activePreset"
          :allow-preview="!isVideo"
          @preview="lightboxOpen = true"
          @select-preset="setPresetFraction"
          @remove="deleteNode()"
        />

        <template v-if="isEditable">
          <div
            v-for="[side, sign] in IMAGE_RESIZE_SIDES"
            :key="side"
            contenteditable="false"
            class="group/handle space-image-handle absolute inset-y-4 flex w-3 cursor-ew-resize items-center justify-center opacity-0 transition-opacity duration-150 group-hover:opacity-100"
            :class="[side, selected && 'opacity-100']"
            @pointerdown="startResize($event, sign)"
            @dblclick.stop="setPresetFraction(1)"
          >
            <span
              class="h-11 w-[5px] rounded-full bg-neutral-950/55 shadow-sm ring-1 ring-white/80 transition-all duration-150 group-hover/handle:w-[6px] group-hover/handle:bg-neutral-950/75"
            />
          </div>
          <div
            ref="badgeRef"
            contenteditable="false"
            class="space-image-badge pointer-events-none absolute bottom-3 left-1/2 -translate-x-1/2 rounded-md bg-neutral-950/82 px-2 py-0.5 text-[11px] font-medium text-white tabular-nums opacity-0 ring-1 ring-white/15"
          />
        </template>
      </div>

      <figcaption
        v-if="showCaption"
        contenteditable="false"
        data-image-caption
        class="w-full px-4"
        :class="hasCaption ? 'mt-2' : 'absolute inset-x-0 top-full z-10 pt-1.5'"
      >
        <textarea
          ref="captionRef"
          rows="1"
          data-testid="image-caption-input"
          :value="altText"
          :disabled="!isEditable"
          :aria-label="t('spaces.richtext.image.caption')"
          :placeholder="t('spaces.richtext.image.captionPlaceholder')"
          class="block w-full resize-none overflow-hidden rounded-md bg-transparent px-2 py-1 text-center text-[13px] leading-[1.55] text-fg-secondary caret-fg-accent transition-[background-color,box-shadow,color] duration-150 outline-none select-text placeholder:text-fg-tertiary/65 focus:bg-surface-muted/70 focus:text-fg-primary focus:ring-1 focus:ring-fg-accent/35 focus:placeholder:opacity-0"
          :class="isEditable ? 'cursor-text hover:bg-surface-muted/50' : 'cursor-default'"
          @mousedown.stop
          @click.stop
          @focus="editingCaption = true"
          @blur="editingCaption = false"
          @input="updateCaption"
          @keydown="onCaptionKeydown"
        />
      </figcaption>
    </div>

    <SpaceTiptapImageLightbox
      v-if="!isVideo"
      :open="lightboxOpen"
      :src="resolvedSrc"
      :caption="altText"
      @close="lightboxOpen = false"
    />
  </NodeViewWrapper>
</template>
