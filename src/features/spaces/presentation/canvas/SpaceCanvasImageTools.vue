<script setup lang="ts">
import {
  Crop,
  Eraser,
  FlipHorizontal2,
  FlipVertical2,
  RotateCw,
  SlidersHorizontal,
  Undo2,
} from "@lucide/vue";
import { ToolbarButton } from "reka-ui";
import { computed, reactive, watch } from "vue";
import { useI18n } from "vue-i18n";

import UiButton from "../../../../shared/ui/UiButton.vue";
import UiIconButton from "../../../../shared/ui/UiIconButton.vue";
import UiPopover from "../../../../shared/ui/UiPopover.vue";
import UiSlider from "../../../../shared/ui/UiSlider.vue";
import { centeredCrop, defaultImageEdit, nextQuarterTurn } from "../../image-editing/image-edit";
import { useSpaceStore } from "../../session/space-store";
import type { useCanvasSelectionActions } from "./use-canvas-selection-actions";

const props = defineProps<{ actions: ReturnType<typeof useCanvasSelectionActions> }>();
const store = useSpaceStore();
const { t } = useI18n();
const edit = computed(() => props.actions.imageEdit.value!);
const node = computed(() => props.actions.selectedImageNode.value!);
const adjustments = reactive({ brightness: 0, contrast: 0, saturation: 0 });
watch(
  () => edit.value.adjustments,
  (value) => Object.assign(adjustments, value),
  { immediate: true },
);

function patch(patchValue: Partial<typeof edit.value>) {
  props.actions.setImageEdit({ ...edit.value, ...patchValue });
}

function crop(aspect?: number) {
  const sourceAspect = (node.value.data.intrinsicWidth ?? 1) / (node.value.data.intrinsicHeight ?? 1);
  const quarterTurn = edit.value.rotation === 90 || edit.value.rotation === 270;
  patch({
    crop: aspect ? centeredCrop(sourceAspect, quarterTurn ? 1 / aspect : aspect) : defaultImageEdit().crop,
  });
}

function undoEraserStroke() {
  patch({ eraserStrokes: edit.value.eraserStrokes.slice(0, -1) });
}
</script>

<template>
  <UiPopover side="top">
    <template #trigger>
      <ToolbarButton as-child>
        <UiIconButton :label="t('spaces.media.crop')" size="sm"
          ><Crop class="size-(--knovum-icon-size-sm)"
        /></UiIconButton>
      </ToolbarButton>
    </template>
    <div class="grid w-44 grid-cols-2 gap-1">
      <UiButton size="sm" variant="ghost" @click="crop()">{{ t("spaces.media.original") }}</UiButton>
      <UiButton size="sm" variant="ghost" @click="crop(1)">1:1</UiButton>
      <UiButton size="sm" variant="ghost" @click="crop(4 / 3)">4:3</UiButton>
      <UiButton size="sm" variant="ghost" @click="crop(16 / 9)">16:9</UiButton>
    </div>
  </UiPopover>
  <ToolbarButton as-child>
    <UiIconButton
      :label="t('spaces.media.rotateClockwise')"
      size="sm"
      @click="patch({ rotation: nextQuarterTurn(edit.rotation) })"
    >
      <RotateCw class="size-(--knovum-icon-size-sm)" />
    </UiIconButton>
  </ToolbarButton>
  <ToolbarButton as-child>
    <UiIconButton
      :label="t('spaces.media.flipHorizontal')"
      size="sm"
      @click="patch({ flipHorizontal: !edit.flipHorizontal })"
    >
      <FlipHorizontal2 class="size-(--knovum-icon-size-sm)" />
    </UiIconButton>
  </ToolbarButton>
  <ToolbarButton as-child>
    <UiIconButton
      :label="t('spaces.media.flipVertical')"
      size="sm"
      @click="patch({ flipVertical: !edit.flipVertical })"
    >
      <FlipVertical2 class="size-(--knovum-icon-size-sm)" />
    </UiIconButton>
  </ToolbarButton>
  <ToolbarButton as-child>
    <UiIconButton
      :label="t('spaces.media.eraser')"
      size="sm"
      :active="store.imageEraserNodeId === node.id"
      @click="store.toggleImageEraser(node.id)"
      ><Eraser class="size-(--knovum-icon-size-sm)"
    /></UiIconButton>
  </ToolbarButton>
  <ToolbarButton v-if="edit.eraserStrokes.length" as-child>
    <UiIconButton :label="t('spaces.media.undoEraser')" size="sm" @click="undoEraserStroke"
      ><Undo2 class="size-(--knovum-icon-size-sm)"
    /></UiIconButton>
  </ToolbarButton>
  <UiPopover side="top" align="end" class="w-64 p-3">
    <template #trigger>
      <ToolbarButton as-child>
        <UiIconButton :label="t('spaces.media.adjustImage')" size="sm"
          ><SlidersHorizontal class="size-(--knovum-icon-size-sm)"
        /></UiIconButton>
      </ToolbarButton>
    </template>
    <div class="space-y-2 text-caption">
      <label class="grid grid-cols-[4.5rem_1fr] items-center gap-2"
        ><span>{{ t("spaces.media.brightness") }}</span
        ><UiSlider
          v-model="adjustments.brightness"
          :label="t('spaces.media.brightness')"
          :min="-100"
          :max="100"
      /></label>
      <label class="grid grid-cols-[4.5rem_1fr] items-center gap-2"
        ><span>{{ t("spaces.media.contrast") }}</span
        ><UiSlider v-model="adjustments.contrast" :label="t('spaces.media.contrast')" :min="-100" :max="100"
      /></label>
      <label class="grid grid-cols-[4.5rem_1fr] items-center gap-2"
        ><span>{{ t("spaces.media.saturation") }}</span
        ><UiSlider
          v-model="adjustments.saturation"
          :label="t('spaces.media.saturation')"
          :min="-100"
          :max="100"
      /></label>
      <div class="flex justify-end gap-1 pt-1">
        <UiButton
          size="sm"
          variant="ghost"
          @click="Object.assign(adjustments, defaultImageEdit().adjustments)"
          >{{ t("common.reset") }}</UiButton
        >
        <UiButton size="sm" variant="primary" @click="patch({ adjustments: { ...adjustments } })">{{
          t("common.apply")
        }}</UiButton>
      </div>
    </div>
  </UiPopover>
</template>
