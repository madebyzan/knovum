<script setup lang="ts">
import { Layer as KonvaLayer, Rect as KonvaRect } from "vue-konva/core";

import { SECTION_HEADER_HEIGHT, type ProjectedSection } from "../../document/structure-scene";
import type { CanvasPalette } from "./use-canvas-palette";

const props = defineProps<{
  palette: CanvasPalette;
  sections: readonly ProjectedSection[];
}>();

function fill(section: ProjectedSection) {
  const appearance = section.layout.appearance;
  if (appearance?.customFill) return appearance.customFill;
  if (appearance?.fill) return props.palette.contentFill[appearance.fill];
  return props.palette.panel;
}
</script>

<template>
  <KonvaLayer v-if="sections.length" :config="{ id: 'canvas-section-background-layer', listening: false }">
    <KonvaRect
      v-for="section in sections"
      :key="section.section.id"
      :config="{
        id: `canvas-section-background-${section.section.id}`,
        x: section.bounds.left,
        y: section.bounds.top,
        width: section.bounds.right - section.bounds.left,
        height:
          section.layout.collapsed && !section.temporarilyExpanded
            ? SECTION_HEADER_HEIGHT
            : section.bounds.bottom - section.bounds.top,
        fill: fill(section),
        opacity: 0.26,
        cornerRadius: 8,
        listening: false,
        perfectDrawEnabled: false,
      }"
    />
  </KonvaLayer>
</template>
