<script setup lang="ts">
import type { CanvasViewport } from "../../contract/canvas-editor-port";
import CanvasGroupScopeBar from "./CanvasGroupScopeBar.vue";
import CanvasStructureRenameOverlay from "./CanvasStructureRenameOverlay.vue";

defineProps<{
  groupScopeTitles: readonly string[];
  renameTarget:
    | {
        structureNodeId: string;
        title: string;
        bounds: { left: number; top: number; right: number; bottom: number };
      }
    | undefined;
  renameError: string | undefined;
  viewport: CanvasViewport;
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
    :title="renameTarget.title"
    :viewport="viewport"
    :error="renameError"
    @cancel="emit('cancelRename')"
    @save="emit('saveRename', $event)"
  />
</template>
