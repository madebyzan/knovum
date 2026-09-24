<script setup lang="ts">
import { computed, type CSSProperties } from "vue";
import { useI18n } from "vue-i18n";

import { useImageEditSession } from "../../../image-editing/image-edit-session";
import type { SpaceImagePayload } from "../../../model/nodes/node-payload";
import { nodePayloadWork } from "./canvas-node-work";
import SpaceCanvasProcessingState from "./SpaceCanvasProcessingState.vue";
import SpaceCanvasWorkDots from "./SpaceCanvasWorkDots.vue";

const props = defineProps<{
  id: string;
  data: SpaceImagePayload;
  loadState: "idle" | "loading" | "ready" | "failed";
  imageRect: { left: number; top: number; width: number; height: number } | undefined;
}>();
const { t } = useI18n();
const session = useImageEditSession();
const work = computed(() => nodePayloadWork(props.data));
const unavailableWork = computed(() => {
  if (props.loadState === "failed")
    return { phase: "failed", labelKey: "spaces.processing.imageUnavailable" };
  if (props.loadState === "loading") return { phase: "active", labelKey: "spaces.processing.loadingImage" };
  return work.value?.surface === "content" ? work.value : undefined;
});
const applyingRegion = computed<CSSProperties | undefined>(() => {
  const active = session.active;
  const rect = props.imageRect;
  if (!session.applying || active?.nodeId !== props.id || !rect) return undefined;
  const crop = active.tool === "crop" ? active.draft.crop : { x: 0, y: 0, width: 1, height: 1 };
  return {
    left: `${rect.left + crop.x * rect.width}px`,
    top: `${rect.top + crop.y * rect.height}px`,
    width: `${crop.width * rect.width}px`,
    height: `${crop.height * rect.height}px`,
  };
});
</script>

<template>
  <div v-if="unavailableWork" class="pointer-events-none absolute inset-0 flex">
    <SpaceCanvasProcessingState
      :label="t(unavailableWork.labelKey)"
      :failed="unavailableWork.phase === 'failed'"
    />
  </div>
  <div v-if="applyingRegion" class="pointer-events-none absolute overflow-hidden" :style="applyingRegion">
    <SpaceCanvasWorkDots placement="selection" />
  </div>
</template>
