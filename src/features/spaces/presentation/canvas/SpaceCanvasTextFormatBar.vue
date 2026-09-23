<script setup lang="ts">
import UiSeparator from "../../../../shared/ui/UiSeparator.vue";
import type { CanvasNodeAppearance } from "../../canvas/contract/canvas-document";
import SpaceCanvasColorPicker from "./SpaceCanvasColorPicker.vue";
import SpaceCanvasTypographyPanel from "./SpaceCanvasTypographyPanel.vue";

defineProps<{ kind: "note" | "text"; appearance?: CanvasNodeAppearance; showSeparator?: boolean }>();
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
  <UiSeparator v-if="showSeparator !== false" orientation="vertical" class="mx-0.5 h-4" />
</template>
