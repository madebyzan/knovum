<script setup lang="ts">
import { useImage } from "@vueuse/core";
import { computed } from "vue";
import { useI18n } from "vue-i18n";
import { Rect as KonvaRect } from "vue-konva/core";

import { versionedMediaSource } from "../../../media/media-element-source";
import { SECTION_HEADER_HEIGHT } from "../../../model/structure/section-metrics";
import type { ProjectedSection } from "../../document/structure-scene";

const props = defineProps<{ section: ProjectedSection; path: string }>();
const emit = defineEmits<{ error: [error: Error] }>();
const { t } = useI18n();
const { state: image } = useImage(() => ({ src: versionedMediaSource(props.path)! }), {
  onError: () => emit("error", new Error(t("spaces.canvas.sections.backgroundFailed"))),
});
const config = computed(() => {
  const width = props.section.layout.size.width;
  const height = props.section.layout.size.height;
  const style = props.section.layout.appearance?.backgroundStyle ?? "cover";
  const source = image.value;
  if (!source) return undefined;
  const scale = patternScale(style, width / source.naturalWidth, height / source.naturalHeight);
  return {
    x: props.section.bounds.left,
    y: props.section.bounds.top,
    width,
    height:
      props.section.layout.collapsed && !props.section.temporarilyExpanded ? SECTION_HEADER_HEIGHT : height,
    fillPatternImage: source,
    fillPatternScale: { x: scale, y: scale },
    fillPatternX: style === "repeat" ? 0 : (width - source.naturalWidth * scale) / 2,
    fillPatternY: style === "repeat" ? 0 : (height - source.naturalHeight * scale) / 2,
    fillPatternRepeat: style === "repeat" ? "repeat" : "no-repeat",
    opacity: 0.26,
    cornerRadius: 12,
    listening: false,
    perfectDrawEnabled: false,
  };
});
function patternScale(style: "cover" | "ratio" | "repeat", horizontal: number, vertical: number) {
  if (style === "repeat") return 1;
  return style === "ratio" ? Math.min(horizontal, vertical) : Math.max(horizontal, vertical);
}
</script>

<template>
  <KonvaRect v-if="config" :config="config" />
</template>
