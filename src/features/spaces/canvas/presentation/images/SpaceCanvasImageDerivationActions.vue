<script setup lang="ts">
import { Eraser, MousePointer2, ScanLine, Sticker } from "@lucide/vue";

import type { ImageExecutionTarget } from "../../../contracts/generated/ImageExecutionTarget";
import SpaceImageLayersPopover from "../../../image-editing/presentation/SpaceImageLayersPopover.vue";
import type { ImageLayersChoiceGroup } from "../../../knowledge-derivations/image-layers-models";
import type { ImageActionAvailability } from "./image-action-availability";
import { DERIVATION_LABEL_KEY } from "./image-derivation-actions";
import SpaceCanvasImageActionTrigger from "./SpaceCanvasImageActionTrigger.vue";
import SpaceImageDirectAction from "./SpaceImageDirectAction.vue";
import type { DirectImageDerivation } from "./use-image-node-bar-state";

/**
 * The decomposition family in reading order: lift the background away, erase a brushed area, pick
 * one object by hand, let the whole picture come apart into its depth layers, or cut a sheet of
 * stickers into one sticker each.
 */
const props = defineProps<{
  availabilityByDerivation: Record<DirectImageDerivation, ImageActionAvailability>;
  reasonByDerivation: Record<DirectImageDerivation, string>;
  removalAvailability: ImageActionAvailability;
  removalReason: string;
  selectionAvailability: ImageActionAvailability;
  selectionReason: string;
  compact?: boolean;
  menuItem?: boolean;
  layersGroups?: ImageLayersChoiceGroup[] | undefined;
}>();
const emit = defineEmits<{
  runDirect: [actionId: DirectImageDerivation, execution?: ImageExecutionTarget];
  beginRemoval: [];
  beginSelection: [];
  runLayers: [selections: Record<string, string>];
  cancelLayers: [];
}>();
const ACTIONS = [
  {
    key: "remove-object",
    icon: Eraser,
    labelKey: "spaces.media.imageEditor.removeObject.tool",
    availability: () => props.removalAvailability,
    reason: () => props.removalReason,
    begin: () => emit("beginRemoval"),
  },
  {
    key: "select-object",
    icon: MousePointer2,
    labelKey: "spaces.media.imageEditor.selection.title",
    availability: () => props.selectionAvailability,
    reason: () => props.selectionReason,
    begin: () => emit("beginSelection"),
  },
] as const;
const STICKER_ACTION = {
  key: "slice-stickers",
  icon: Sticker,
  labelKey: DERIVATION_LABEL_KEY["slice-sticker-sheet"],
  availability: () => props.availabilityByDerivation["slice-sticker-sheet"],
  reason: () => props.reasonByDerivation["slice-sticker-sheet"],
  begin: () => emit("runDirect", "slice-sticker-sheet"),
} as const;
</script>

<template>
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
    v-for="action in ACTIONS"
    :key="action.key"
    :availability="action.availability()"
    :reason="action.reason()"
    :icon="action.icon"
    :label-key="action.labelKey"
    :compact="compact"
    :menu-item="menuItem"
    @begin="action.begin()"
  />
  <SpaceImageLayersPopover
    :availability="availabilityByDerivation['split-image-layers']"
    :reason="reasonByDerivation['split-image-layers']"
    :compact="compact"
    :menu-item="menuItem"
    :open="Boolean(layersGroups?.length)"
    :groups="layersGroups"
    @invoke="emit('runDirect', 'split-image-layers')"
    @run="emit('runLayers', $event)"
    @update:open="!$event && emit('cancelLayers')"
  />
  <SpaceCanvasImageActionTrigger
    :availability="STICKER_ACTION.availability()"
    :reason="STICKER_ACTION.reason()"
    :icon="STICKER_ACTION.icon"
    :label-key="STICKER_ACTION.labelKey"
    :compact="compact"
    :menu-item="menuItem"
    @begin="STICKER_ACTION.begin()"
  />
</template>
