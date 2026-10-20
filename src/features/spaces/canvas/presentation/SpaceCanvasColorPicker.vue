<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";

import {
  CONTENT_FILL_PRESETS,
  CONTENT_TEXT_COLOR_NAMES,
} from "../../model/structure/schema/content-appearance";
import type { CanvasNodeAppearance } from "../schema/document";
import { CONTENT_FILL_COLORS, CONTENT_TEXT_COLORS } from "./canvas-content-colors";
import SpaceCanvasColorSwatches from "./SpaceCanvasColorSwatches.vue";

const props = defineProps<{ kind: "note" | "text"; appearance?: CanvasNodeAppearance; compact: boolean }>();
const emit = defineEmits<{ update: [patch: Partial<CanvasNodeAppearance>] }>();
const { t } = useI18n();
const label = computed(() =>
  t(props.kind === "note" ? "spaces.canvas.noteBackground" : "spaces.canvas.textColor"),
);
const selected = computed(() =>
  props.kind === "note" ? (props.appearance?.fill ?? "yellow") : (props.appearance?.textColor ?? "ink"),
);
const options = computed(() =>
  props.kind === "note"
    ? CONTENT_FILL_PRESETS.map((value) => ({
        value,
        color: CONTENT_FILL_COLORS[value],
        label: t(`spaces.canvas.noteFills.${value}`),
      }))
    : CONTENT_TEXT_COLOR_NAMES.map((value) => ({
        value,
        color: CONTENT_TEXT_COLORS[value],
        label: t(`spaces.canvas.textColors.${value}`),
      })),
);

function select(value: string) {
  if (props.kind === "note") {
    const fill = CONTENT_FILL_PRESETS.find((preset) => preset === value);
    if (fill) emit("update", { fill, customFill: undefined });
    return;
  }
  const textColor = CONTENT_TEXT_COLOR_NAMES.find((color) => color === value);
  if (textColor) emit("update", { textColor });
}
</script>

<template>
  <SpaceCanvasColorSwatches
    :label="label"
    labeled
    :compact="compact"
    :options="options"
    :selected="selected"
    @select="select"
  />
</template>
