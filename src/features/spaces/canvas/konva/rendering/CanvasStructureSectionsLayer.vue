<script setup lang="ts">
import { Layer as KonvaLayer, Rect as KonvaRect } from "vue-konva/core";

import { SECTION_HEADER_HEIGHT } from "../../../model/structure/section-metrics";
import type { ProjectedSection } from "../../document/structure-scene";
import CanvasSectionBackgroundImage from "./CanvasSectionBackgroundImage.vue";
import type { CanvasPalette } from "./use-canvas-palette";

const props = defineProps<{
  palette: CanvasPalette;
  sections: readonly ProjectedSection[];
}>();

const emit = defineEmits<{ error: [error: Error] }>();

function fill(section: ProjectedSection) {
  const appearance = section.layout.appearance;
  if (appearance?.customFill) return appearance.customFill;
  if (appearance?.fill) return props.palette.contentFill[appearance.fill];
  return props.palette.panel;
}
</script>

<template>
  <KonvaLayer v-if="sections.length" :config="{ id: 'canvas-section-background-layer', listening: false }">
    <template v-for="section in sections" :key="section.section.id">
      <KonvaRect
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
          opacity: 0.22,
          cornerRadius: 12,
          listening: false,
          perfectDrawEnabled: false,
        }"
      />
      <CanvasSectionBackgroundImage
        v-if="section.layout.appearance?.background"
        :section="section"
        :path="section.layout.appearance.background"
        @error="emit('error', $event)"
      />
    </template>
  </KonvaLayer>
</template>
