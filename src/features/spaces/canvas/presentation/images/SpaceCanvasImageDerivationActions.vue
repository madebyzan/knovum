<script setup lang="ts">
import { Eraser, Layers3, MousePointer2, ScanLine } from "@lucide/vue";

import type { ImageActionAvailability } from "./image-action-availability";
import SpaceCanvasImageActionTrigger from "./SpaceCanvasImageActionTrigger.vue";
import type { DirectImageDerivation } from "./use-image-node-bar-state";

/**
 * The decomposition family in reading order: lift the background away, erase a brushed area, pick
 * one object by hand, or let the whole picture come apart into layers.
 */
const props = defineProps<{
  availability: Record<DirectImageDerivation, ImageActionAvailability>;
  reason: Record<DirectImageDerivation, string>;
  removalAvailability: ImageActionAvailability;
  removalReason: string;
  selectionAvailability: ImageActionAvailability;
  selectionReason: string;
  compact?: boolean;
  menuItem?: boolean;
}>();
const emit = defineEmits<{
  runDirect: [actionId: DirectImageDerivation];
  beginRemoval: [];
  beginSelection: [];
}>();
const ACTIONS = [
  {
    key: "remove-image-background",
    icon: ScanLine,
    labelKey: "spaces.creation.image.removeBackground",
    availability: () => props.availability["remove-image-background"],
    reason: () => props.reason["remove-image-background"],
    begin: () => emit("runDirect", "remove-image-background"),
  },
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
  {
    key: "split-layers",
    icon: Layers3,
    labelKey: "spaces.creation.image.splitLayers",
    availability: () => props.availability["split-image-layers"],
    reason: () => props.reason["split-image-layers"],
    begin: () => emit("runDirect", "split-image-layers"),
  },
] as const;
</script>

<template>
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
</template>
