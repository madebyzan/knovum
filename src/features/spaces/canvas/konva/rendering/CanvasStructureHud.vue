<script setup lang="ts">
import type { CanvasCamera } from "../viewport/canvas-camera";
import CanvasGroupScopeBar from "./CanvasGroupScopeBar.vue";
import CanvasStructureRenameOverlay from "./CanvasStructureRenameOverlay.vue";

defineProps<{
  camera: CanvasCamera;
  groupScopeTitles: readonly string[];
  renameTarget:
    | {
        structureNodeId: string;
        title: string;
        bounds: { left: number; top: number; right: number; bottom: number };
      }
    | undefined;
  renameError: string | undefined;
  zoom: number;
}>();
const emit = defineEmits<{
  back: [];
  cancelRename: [];
  saveRename: [title: string];
}>();
</script>

<template>
  <CanvasGroupScopeBar v-if="groupScopeTitles.length" :titles="groupScopeTitles" @back="emit('back')" />
  <CanvasStructureRenameOverlay
    v-if="renameTarget"
    :key="renameTarget.structureNodeId"
    :bounds="renameTarget.bounds"
    :camera="camera"
    :title="renameTarget.title"
    :zoom="zoom"
    :error="renameError"
    @cancel="emit('cancelRename')"
    @save="emit('saveRename', $event)"
  />
</template>
