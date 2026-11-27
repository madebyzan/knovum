<script setup lang="ts">
import { Eraser, Layers3, MousePointer2, ScanLine, Sticker } from "@lucide/vue";

import type { ImageExecutionTarget } from "../../../contracts/generated/ImageExecutionTarget";
import type { ImageActionAvailability } from "./image-action-availability";
import { DERIVATION_LABEL_KEY } from "./image-derivation-actions";
import SpaceCanvasImageActionTrigger from "./SpaceCanvasImageActionTrigger.vue";
import SpaceImageDirectAction from "./SpaceImageDirectAction.vue";
import type { DirectImageDerivation } from "./use-image-node-bar-state";

/**
 * Image AI utilities split by frequency: remove-background and object-removal stay on the primary
 * toolbar, while lower-frequency decomposition actions (cutout, layers, stickers) live in the
 * overflow menu (which also absorbs the primary pair in minimal width mode).
 */
defineProps<{
  availabilityByDerivation: Record<DirectImageDerivation, ImageActionAvailability>;
  reasonByDerivation: Record<DirectImageDerivation, string>;
  removalAvailability: ImageActionAvailability;
  removalReason: string;
  selectionAvailability: ImageActionAvailability;
  selectionReason: string;
  compact?: boolean;
  menuItem?: boolean;
  includePrimary?: boolean;
}>();
const emit = defineEmits<{
  runDirect: [actionId: DirectImageDerivation, execution?: ImageExecutionTarget];
  beginRemoval: [];
  beginSelection: [];
}>();
</script>

<template>
  <template v-if="!menuItem || includePrimary">
    <SpaceImageDirectAction
      capability="imageSegmentation"
      :availability="availabilityByDerivation['remove-image-background']"
      :reason="reasonByDerivation['remove-image-background']"
      :icon="ScanLine"
      :label-key="DERIVATION_LABEL_KEY['remove-image-background']"
      :compact="compact"
      :menu-item="menuItem"
      @run="emit('runDirect', 'remove-image-background', $event)"
    />
    <SpaceCanvasImageActionTrigger
      :availability="removalAvailability"
      :reason="removalReason"
      :icon="Eraser"
      label-key="spaces.media.imageEditor.removeObject.tool"
      :compact="compact"
      :menu-item="menuItem"
      @begin="emit('beginRemoval')"
    />
  </template>
  <template v-if="menuItem">
    <SpaceCanvasImageActionTrigger
      :availability="selectionAvailability"
      :reason="selectionReason"
      :icon="MousePointer2"
      label-key="spaces.media.imageEditor.selection.title"
      menu-item
      @begin="emit('beginSelection')"
    />
    <SpaceCanvasImageActionTrigger
      :availability="availabilityByDerivation['split-image-layers']"
      :reason="reasonByDerivation['split-image-layers']"
      :icon="Layers3"
      :label-key="DERIVATION_LABEL_KEY['split-image-layers']"
      menu-item
      @begin="emit('runDirect', 'split-image-layers')"
    />
    <SpaceCanvasImageActionTrigger
      :availability="availabilityByDerivation['slice-sticker-sheet']"
      :reason="reasonByDerivation['slice-sticker-sheet']"
      :icon="Sticker"
      :label-key="DERIVATION_LABEL_KEY['slice-sticker-sheet']"
      menu-item
      @begin="emit('runDirect', 'slice-sticker-sheet')"
    />
  </template>
</template>
