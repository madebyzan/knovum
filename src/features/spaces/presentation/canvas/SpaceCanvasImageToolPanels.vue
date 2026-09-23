<script setup lang="ts">
import { Crop, Eraser, RotateCw, SlidersHorizontal } from "@lucide/vue";
import { ToolbarButton } from "reka-ui";
import { computed } from "vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";

import UiButton from "../../../../shared/ui/UiButton.vue";
import UiPopover from "../../../../shared/ui/UiPopover.vue";
import { defaultImageEdit, imageEditOf } from "../../image-editing/image-edit";
import { useImageEditSession, type ImageEditTool } from "../../image-editing/image-edit-session";
import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import type { SpaceImagePayload } from "../../model/nodes/node-payload";
import SpaceImageAdjustPanel from "./SpaceImageAdjustPanel.vue";
import SpaceImageCropPanel from "./SpaceImageCropPanel.vue";
import SpaceImageEraserPanel from "./SpaceImageEraserPanel.vue";

const props = defineProps<{ node: SpaceCanvasNode<SpaceImagePayload>; mode: ImageEditTool }>();
const session = useImageEditSession();
const { t } = useI18n();
const edit = computed(() => session.previewFor(props.node.id) ?? imageEditOf(props.node.data));
const disabled = computed(() => session.applying || Boolean(session.active && session.active.tool !== props.mode));
const dimensions = computed(() => {
  const loaded = session.dimensionsFor(props.node.id);
  const width = props.node.data.intrinsicWidth ?? loaded?.width;
  const height = props.node.data.intrinsicHeight ?? loaded?.height;
  return width && height ? { width, height } : undefined;
});

function toolOpen(tool: ImageEditTool) {
  return computed({
    get: () => session.active?.nodeId === props.node.id && session.active.tool === tool,
    set: (open: boolean) => {
      if (open) session.begin(props.node.id, tool);
      else if (session.active?.nodeId === props.node.id && session.active.tool === tool) session.cancel();
    },
  });
}

const adjustOpen = toolOpen("adjust");
const cropOpen = toolOpen("crop");
const eraserOpen = toolOpen("eraser");

function apply() {
  void session.apply().catch((error) => toast.error(String(error)));
}
</script>

<template>
  <UiPopover v-if="mode === 'eraser'" v-model:open="eraserOpen" side="bottom" align="start" persistent class="p-2">
    <template #trigger>
      <ToolbarButton as-child>
        <UiButton variant="ghost" size="sm" class="h-(--knovum-control-height-md) shadow-none" :disabled="disabled">
          <Eraser class="size-(--knovum-icon-size-toolbar)" :stroke-width="1.6" />
          {{ t("spaces.media.eraser") }}
        </UiButton>
      </ToolbarButton>
    </template>
    <SpaceImageEraserPanel
      :diameter="session.brushDiameter"
      :can-undo="edit.eraserStrokes.length > imageEditOf(node.data).eraserStrokes.length"
      :can-apply="edit.eraserStrokes.length !== imageEditOf(node.data).eraserStrokes.length"
      @update-diameter="session.setBrushDiameter"
      @undo="session.undoEraserStroke"
      @cancel="session.cancel"
      @apply="apply"
    />
  </UiPopover>
  <UiPopover v-if="mode === 'adjust'" v-model:open="adjustOpen" side="right" align="start" class="p-3">
    <template #trigger>
      <ToolbarButton as-child>
        <UiButton variant="ghost" size="sm" class="h-(--knovum-control-height-md) shadow-none" :disabled="disabled">
          <SlidersHorizontal class="size-(--knovum-icon-size-toolbar)" :stroke-width="1.6" />
          {{ t("spaces.media.adjustImage") }}
        </UiButton>
      </ToolbarButton>
    </template>
    <SpaceImageAdjustPanel
      :adjustments="edit.adjustments"
      @update="(key, value) => session.update({ adjustments: { ...edit.adjustments, [key]: value } })"
      @reset="session.update({ adjustments: defaultImageEdit().adjustments })"
      @cancel="session.cancel"
      @apply="apply"
    />
  </UiPopover>
  <UiPopover v-if="mode === 'crop' && dimensions" v-model:open="cropOpen" side="right" align="start" persistent class="p-3">
    <template #trigger>
      <ToolbarButton as-child>
        <UiButton variant="ghost" size="sm" class="h-(--knovum-control-height-md) shadow-none" :disabled="disabled">
          <Crop class="size-(--knovum-icon-size-toolbar)" :stroke-width="1.6" />
          {{ t("spaces.media.crop") }}
        </UiButton>
      </ToolbarButton>
    </template>
    <SpaceImageCropPanel
      :crop="edit.crop"
      :source-width="dimensions.width"
      :source-height="dimensions.height"
      :locked-aspect="session.cropAspectRatio"
      @update="(crop) => session.update({ crop })"
      @update-locked-aspect="session.setCropAspectRatio"
      @cancel="session.cancel"
      @apply="apply"
    />
  </UiPopover>
  <ToolbarButton v-if="mode === 'transform'" as-child>
    <UiButton
      variant="ghost"
      size="sm"
      class="h-(--knovum-control-height-md) shadow-none"
      :disabled="disabled"
      @click="session.begin(node.id, 'transform')"
    >
      <RotateCw class="size-(--knovum-icon-size-toolbar)" :stroke-width="1.6" />
      {{ t("spaces.media.transformImage") }}
    </UiButton>
  </ToolbarButton>
</template>
