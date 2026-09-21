<script setup lang="ts">
import { AlertCircle, X } from "@lucide/vue";
import { computed } from "vue";
import { useI18n } from "vue-i18n";

import UiIconButton from "../../../../shared/ui/UiIconButton.vue";
import { useCanvasEditorSession } from "../../canvas/editor-session";
import type { SpaceUrlCaptureProjection } from "../../sources/url-capture-projection";
import SpaceEmptyState from "../SpaceEmptyState.vue";
import SpaceCanvasMediaImport from "./SpaceCanvasMediaImport.vue";
import SpaceCanvasSceneFeedback from "./SpaceCanvasSceneFeedback.vue";

const props = defineProps<{
  canvasToViewportPoint(point: { x: number; y: number }): { x: number; y: number };
  captures: readonly SpaceUrlCaptureProjection[];
  empty: boolean;
  error?: string;
  fileHover: boolean;
  fileHoverPosition?: { x: number; y: number };
  spaceId: string;
}>();
const emit = defineEmits<{
  dismissError: [];
  dismissCapture: [id: string];
  retryCapture: [capture: SpaceUrlCaptureProjection];
}>();
const { t } = useI18n();
const editorSession = useCanvasEditorSession();
const visibleCaptures = computed(() => props.captures.filter((capture) => capture.spaceId === props.spaceId));

function captureStyle(capture: SpaceUrlCaptureProjection) {
  const position = props.canvasToViewportPoint(capture.position);
  const zoom = editorSession.viewport.value?.zoom ?? 1;
  return {
    transform: `translate(${position.x}px, ${position.y}px) scale(${Math.min(1, Math.max(0.75, zoom))})`,
  };
}
</script>

<template>
  <SpaceCanvasSceneFeedback />
  <SpaceEmptyState v-if="empty" />
  <TransitionGroup
    tag="div"
    class="contents"
    enter-active-class="transition-opacity duration-150 ease-standard motion-reduce:transition-none"
    enter-from-class="opacity-0"
    leave-active-class="transition-opacity duration-100 ease-standard motion-reduce:transition-none"
    leave-to-class="opacity-0"
  >
    <SpaceCanvasMediaImport
      v-for="capture in visibleCaptures"
      :key="capture.id"
      class="pointer-events-auto absolute top-0 left-0 z-20 origin-top-left"
      :style="captureStyle(capture)"
      :capture="capture"
      @dismiss="emit('dismissCapture', capture.id)"
      @retry="emit('retryCapture', capture)"
    />
  </TransitionGroup>
  <div
    v-if="fileHover && fileHoverPosition"
    class="pointer-events-none fixed z-40 -translate-y-full pb-3 pl-3"
    :style="{ left: `${fileHoverPosition.x}px`, top: `${fileHoverPosition.y}px` }"
  >
    <div class="rounded-control border border-border bg-overlay px-3 py-1.5 text-label font-medium">
      {{ t("spaces.canvas.actions.dropToSave") }}
    </div>
  </div>
  <div
    v-if="error"
    class="pointer-events-auto absolute top-4 left-1/2 flex max-w-sm -translate-x-1/2 items-center gap-2 rounded-control border border-border bg-overlay py-1 pr-0.5 pl-2.5 text-caption text-foreground-secondary"
    role="alert"
    aria-atomic="true"
  >
    <AlertCircle class="size-(--knovum-icon-size-xs) shrink-0" :stroke-width="1.6" />
    <span class="line-clamp-2 min-w-0 flex-1 wrap-break-word">{{ error }}</span>
    <UiIconButton :label="t('common.close')" size="sm" @click="emit('dismissError')">
      <X class="size-(--knovum-icon-size-xs)" />
    </UiIconButton>
  </div>
</template>
