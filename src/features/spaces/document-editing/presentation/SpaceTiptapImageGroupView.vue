<script setup lang="ts">
import { convertFileSrc } from "@tauri-apps/api/core";
import { NodeViewWrapper, type NodeViewProps } from "@tiptap/vue-3";
import { computed, onBeforeUnmount, shallowRef, watch } from "vue";

import {
  normalizeImageGroupLayout,
  resolveImageGroupColumns,
  type ImageGroupLayout,
  type LightboxImageItem,
} from "../../../../shared/ui/image-group-layout";
import {
  removeGroupImage,
  reorderGroupImage,
  splitImageGroupIntoParagraphs,
  updateGroupImageAlt,
} from "../tiptap-image-group-actions";
import { chooseSpaceTiptapImages } from "../tiptap-images";
import SpaceTiptapImageCompare from "./SpaceTiptapImageCompare.vue";
import SpaceTiptapImageGroupBar from "./SpaceTiptapImageGroupBar.vue";
import SpaceTiptapImageGroupTile from "./SpaceTiptapImageGroupTile.vue";
import SpaceTiptapImageLightbox from "./SpaceTiptapImageLightbox.vue";

const DRAG_THRESHOLD_PX = 6;
const props = defineProps<NodeViewProps>();

const isEditable = shallowRef(props.editor.isEditable);
const editingIndex = shallowRef<number | null>(null);
const lightboxOpen = shallowRef(false);
const lightboxIndex = shallowRef(0);
const dragSourceIndex = shallowRef<number | null>(null);
const dragTargetIndex = shallowRef<number | null>(null);
let pendingDrag: { index: number; startX: number; startY: number; active: boolean } | null = null;
let suppressNextClick = false;

watch(
  () => props.editor,
  (editor, _prev, cleanup) => {
    const syncEditable = () => (isEditable.value = editor.isEditable);
    syncEditable();
    editor.on("update", syncEditable);
    cleanup(() => editor.off("update", syncEditable));
  },
  { immediate: true },
);

const layout = computed<ImageGroupLayout>(() => normalizeImageGroupLayout(props.node.attrs.layout));

const images = computed<readonly LightboxImageItem[]>(() => {
  const list: LightboxImageItem[] = [];
  props.node.forEach((child) => {
    if (child.type.name !== "image") return;
    const assetPath = child.attrs.assetPath;
    const src =
      typeof assetPath === "string" && assetPath
        ? convertFileSrc(assetPath)
        : typeof child.attrs.src === "string"
          ? child.attrs.src
          : "";
    list.push({ src, caption: typeof child.attrs.alt === "string" ? child.attrs.alt : "" });
  });
  return list;
});

const columns = computed(() => resolveImageGroupColumns(layout.value, images.value.length));
const useCompare = computed(() => layout.value === "compare" && images.value.length >= 2);
const gridItems = computed(() => (useCompare.value ? images.value.slice(2) : images.value));
const gridOffset = computed(() => (useCompare.value ? 2 : 0));

function openLightbox(index: number) {
  if (suppressNextClick) {
    suppressNextClick = false;
    return;
  }
  lightboxIndex.value = index;
  lightboxOpen.value = true;
}

function clearTileDrag() {
  globalThis.removeEventListener("pointermove", onTilePointerMove);
  globalThis.removeEventListener("pointerup", onTilePointerUp);
  globalThis.removeEventListener("pointercancel", clearTileDrag);
  pendingDrag = null;
  dragSourceIndex.value = null;
  dragTargetIndex.value = null;
}

function onTilePointerMove(event: PointerEvent) {
  if (!pendingDrag) return;
  if (!pendingDrag.active) {
    const moved = Math.hypot(event.clientX - pendingDrag.startX, event.clientY - pendingDrag.startY);
    if (moved < DRAG_THRESHOLD_PX) return;
    pendingDrag.active = true;
    dragSourceIndex.value = pendingDrag.index;
  }
  const hit = document.elementFromPoint(event.clientX, event.clientY)?.closest("[data-image-tile-index]");
  const rawIdx = hit?.getAttribute("data-image-tile-index");
  const idx = rawIdx !== null && rawIdx !== undefined ? Number(rawIdx) : null;
  dragTargetIndex.value = idx !== null && idx !== pendingDrag.index ? idx : null;
}

function onTilePointerUp() {
  const active = pendingDrag?.active ? { from: pendingDrag.index, to: dragTargetIndex.value } : null;
  clearTileDrag();
  if (!active) return;
  suppressNextClick = true;
  if (active.to !== null) reorderGroupImage(props.editor, props.getPos, active.from, active.to);
}

function startTileDrag(index: number, event: PointerEvent) {
  if (!isEditable.value || event.button !== 0 || images.value.length < 2) return;
  pendingDrag = { index, startX: event.clientX, startY: event.clientY, active: false };
  globalThis.addEventListener("pointermove", onTilePointerMove, { passive: true });
  globalThis.addEventListener("pointerup", onTilePointerUp);
  globalThis.addEventListener("pointercancel", clearTileDrag);
}

async function handleAddImages() {
  const pos = props.getPos();
  if (pos === undefined) return;
  props.editor.commands.setTextSelection(pos + props.node.nodeSize);
  await chooseSpaceTiptapImages(props.editor);
}

onBeforeUnmount(clearTileDrag);
</script>

<template>
  <NodeViewWrapper
    as="section"
    data-type="image-group"
    data-testid="tiptap-image-group"
    :data-layout="layout"
    :data-count="images.length"
    :style="{ '--image-group-cols': columns }"
    class="ui-image-group group/img-group relative rounded-xl"
    :class="selected && 'ring-[1.5px] ring-fg-accent/70 ring-offset-3 ring-offset-surface-canvas'"
  >
    <SpaceTiptapImageGroupBar
      v-if="isEditable"
      :layout="layout"
      :image-count="images.length"
      :visible="selected || editingIndex !== null"
      @set-layout="(next) => updateAttributes({ layout: next })"
      @add-images="void handleAddImages()"
      @split-group="splitImageGroupIntoParagraphs(editor, getPos)"
    />

    <SpaceTiptapImageCompare
      v-if="useCompare"
      :before="images[0]!"
      :after="images[1]!"
      @preview="openLightbox"
    />

    <div v-if="gridItems.length" class="ui-image-group-grid" :class="useCompare && 'mt-3'">
      <SpaceTiptapImageGroupTile
        v-for="(item, localIdx) in gridItems"
        :key="`${item.src}-${localIdx + gridOffset}`"
        :item="item"
        :index="localIdx + gridOffset"
        :layout="layout"
        :is-editable="isEditable"
        :editing="editingIndex === localIdx + gridOffset"
        :drag-source="dragSourceIndex === localIdx + gridOffset"
        :drag-target="dragTargetIndex === localIdx + gridOffset"
        @preview="openLightbox(localIdx + gridOffset)"
        @remove="removeGroupImage(editor, getPos, localIdx + gridOffset)"
        @start-edit="editingIndex = localIdx + gridOffset"
        @stop-edit="editingIndex = null"
        @update-alt="(value) => updateGroupImageAlt(editor, getPos, localIdx + gridOffset, value)"
        @drag-start="(event) => startTileDrag(localIdx + gridOffset, event)"
      />
    </div>

    <SpaceTiptapImageLightbox
      :open="lightboxOpen"
      :items="images"
      :initial-index="lightboxIndex"
      @close="lightboxOpen = false"
    />
  </NodeViewWrapper>
</template>
