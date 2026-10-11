<script setup lang="ts">
import type { CanvasNodeAppearance } from "../schema/document";
import SpaceCanvasColorPicker from "./SpaceCanvasColorPicker.vue";
import SpaceCanvasTypographyControls from "./SpaceCanvasTypographyControls.vue";

defineProps<{ kind: "note" | "text"; appearance?: CanvasNodeAppearance; compact: boolean }>();
const emit = defineEmits<{ update: [patch: Partial<CanvasNodeAppearance>] }>();
</script>

<template>
  <SpaceCanvasColorPicker
    :kind="kind"
    :compact="compact"
    v-bind="appearance ? { appearance } : {}"
    @update="emit('update', $event)"
  />
  <SpaceCanvasTypographyControls
    :kind="kind"
    :compact="compact"
    v-bind="appearance ? { appearance } : {}"
    @update="emit('update', $event)"
  />
</template>
