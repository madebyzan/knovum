<script setup lang="ts">
import type { CanvasNodeAppearance } from "../../canvas/contract/canvas-document";
import SpaceCanvasColorPicker from "./SpaceCanvasColorPicker.vue";
import SpaceCanvasTypographyPanel from "./SpaceCanvasTypographyPanel.vue";

defineProps<{ kind: "note" | "text"; appearance?: CanvasNodeAppearance }>();
const emit = defineEmits<{ update: [patch: Partial<CanvasNodeAppearance>] }>();
</script>

<template>
  <SpaceCanvasColorPicker
    :kind="kind"
    v-bind="appearance ? { appearance } : {}"
    @update="emit('update', $event)"
  />
  <SpaceCanvasTypographyPanel
    :kind="kind"
    v-bind="appearance ? { appearance } : {}"
    @update="emit('update', $event)"
  />
</template>
