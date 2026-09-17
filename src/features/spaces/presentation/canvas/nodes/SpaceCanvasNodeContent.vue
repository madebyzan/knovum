<script setup lang="ts">
import { computed } from "vue";

import { spaceAssetSource } from "../../../assets/node-preview-source";
import type { CanvasNodeLayoutV1 } from "../../../canvas/contract/canvas-document";
import type { CanvasContentHeightMeasurement } from "../../../canvas/contract/canvas-editor-port";
import { CANVAS_NODE_HEADER_HEIGHT, PREVIEW_CAPTION_HEIGHT } from "../../../canvas/node-size";
import type { SpaceCanvasNode } from "../../../model/nodes/canvas-node";
import { canvasTextPresentation } from "./canvas-text-appearance";
import { spaceCanvasNodePresentation } from "./space-canvas-node-presentation";
import SpaceCanvasCredential from "./SpaceCanvasCredential.vue";
import SpaceCanvasNodeBody from "./SpaceCanvasNodeBody.vue";
import SpaceCanvasNodeStatus from "./SpaceCanvasNodeStatus.vue";
import SpaceCanvasNote from "./SpaceCanvasNote.vue";
import SpaceCanvasText from "./SpaceCanvasText.vue";
import SpaceCanvasVideoCaption from "./SpaceCanvasVideoCaption.vue";
import SpaceCanvasWebsite from "./SpaceCanvasWebsite.vue";

// Canvas text styles are deliberately loaded for this composition surface.
// oxlint-disable-next-line import/no-unassigned-import
import "./canvas-text-content.css";

const props = defineProps<{
  layout: CanvasNodeLayoutV1;
  node: SpaceCanvasNode;
  detailLevel: "compact" | "summary" | "full";
  interactive: boolean;
  persist: (id: string, data: SpaceCanvasNode["data"]) => Promise<void>;
  fitContentHeight: (id: string, measurement: CanvasContentHeightMeasurement) => void;
}>();
const presentation = computed(() => spaceCanvasNodePresentation(props.node.data));
const isSection = computed(() => props.node.data.kind === "section");
const isText = computed(() => props.node.data.kind === "text");
const isNote = computed(() => props.node.data.kind === "note");
const hasCredentialAction = computed(() => props.node.data.kind === "credential");
const sectionBackground = computed(() => {
  const appearance = props.layout.appearance;
  if (!isSection.value || !appearance?.background) return undefined;
  const mode = appearance.backgroundStyle ?? "cover";
  return {
    backgroundImage: `url("${spaceAssetSource(appearance.background)}")`,
    backgroundPosition: "center",
    backgroundRepeat: mode === "repeat" ? "repeat" : "no-repeat",
    backgroundSize: mode === "ratio" ? "contain" : mode === "cover" ? "cover" : "auto",
  };
});
const contentDetailLevel = computed(() => (props.interactive ? "full" : props.detailLevel));
const interactiveMedia = computed(
  () => props.interactive && (props.node.data.kind === "audio" || props.node.data.kind === "video"),
);
const previewFirst = computed(() => ["audio", "file", "image", "video"].includes(props.node.data.kind));
const usesMultilinePreviewCaption = computed(() => ["file", "image"].includes(props.node.data.kind));
const previewCaptionStyle = computed(() =>
  usesMultilinePreviewCaption.value ? { height: `${PREVIEW_CAPTION_HEIGHT}px` } : undefined,
);
const textPresentation = computed(() =>
  canvasTextPresentation(isNote.value ? "note" : "text", props.layout.appearance),
);
</script>

<template>
  <article
    class="flex size-full min-h-0 flex-col overflow-hidden text-foreground select-none"
    :class="[
      isSection
        ? 'rounded-canvas-object border border-transparent bg-transparent'
        : isText
          ? 'bg-transparent'
          : isNote
            ? 'rounded-sm shadow-canvas-object'
            : 'rounded-canvas-object border border-border bg-panel shadow-canvas-object',
      (isNote || isText) && textPresentation.classes,
      interactive && !interactiveMedia && !hasCredentialAction && 'pointer-events-auto',
    ]"
    :style="[sectionBackground, (isNote || isText) && textPresentation.style]"
    :data-kind="node.data.kind"
    :aria-hidden="interactive ? undefined : true"
    :inert="interactive ? undefined : true"
  >
    <SpaceCanvasCredential
      v-if="node.data.kind === 'credential'"
      :id="node.id"
      :data="node.data"
      :interactive="interactive"
      :persist="persist"
    />
    <SpaceCanvasNote
      v-else-if="node.data.kind === 'note'"
      :id="node.id"
      :data="node.data"
      :interactive="interactive"
      :persist="persist"
      :vertical-class="textPresentation.verticalClass"
      :font-size="textPresentation.fontSize"
      :text-align="textPresentation.textAlign"
    />
    <SpaceCanvasText
      v-else-if="node.data.kind === 'text'"
      :id="node.id"
      :data="node.data"
      :interactive="interactive"
      :persist="persist"
      :vertical-class="textPresentation.verticalClass"
      :fit-content-height="fitContentHeight"
      :font-size="textPresentation.fontSize"
      :frame-width="layout.frame.width"
      :text-align="textPresentation.textAlign"
      :typography-scale="layout.appearance?.typographyScale ?? 1"
    />
    <SpaceCanvasWebsite v-else-if="node.data.kind === 'website'" :id="node.id" :data="node.data" />
    <template v-else>
      <header
        v-if="!previewFirst"
        class="pointer-events-none flex shrink-0 items-center gap-2 border-b border-divider px-3"
        :style="{ height: `${CANVAS_NODE_HEADER_HEIGHT}px` }"
      >
        <component
          :is="presentation.icon"
          class="size-(--knovum-icon-size-sm) shrink-0 text-foreground-secondary"
          :stroke-width="1.55"
        />
        <span class="min-w-0 flex-1">
          <strong class="block truncate text-label font-medium">{{ presentation.title }}</strong>
          <span
            v-if="contentDetailLevel !== 'compact' && presentation.subtitle"
            class="block truncate text-micro text-foreground-tertiary"
            >{{ presentation.subtitle }}</span
          >
        </span>
        <span v-if="node.data.locked" class="text-micro text-foreground-tertiary">Locked</span>
        <SpaceCanvasNodeStatus :id="node.id" :data="node.data" />
      </header>
      <SpaceCanvasNodeBody
        :id="node.id"
        :data="node.data"
        :detail-level="contentDetailLevel"
        :interactive="interactive"
      />
      <SpaceCanvasVideoCaption v-if="node.data.kind === 'video'" :id="node.id" :data="node.data" />
      <footer
        v-if="previewFirst && node.data.kind !== 'audio' && node.data.kind !== 'video'"
        class="pointer-events-none flex shrink-0 items-center gap-2 border-t border-divider px-3 py-1.5"
        :class="!usesMultilinePreviewCaption && 'h-13'"
        :style="previewCaptionStyle"
      >
        <span class="min-w-0 flex-1">
          <strong
            class="wrap-break-word"
            :class="
              usesMultilinePreviewCaption
                ? 'line-clamp-2 text-body leading-snug font-semibold'
                : 'block truncate text-label font-medium'
            "
            >{{ presentation.title }}</strong
          >
          <span
            v-if="contentDetailLevel !== 'compact' && presentation.subtitle"
            class="mt-0.5 block truncate text-caption text-foreground-tertiary"
            >{{ presentation.subtitle }}</span
          >
        </span>
        <SpaceCanvasNodeStatus :id="node.id" :data="node.data" />
      </footer>
      <div v-if="layout.fileSubpath" class="shrink-0 px-3 pb-2">
        <p class="mt-2 truncate text-micro text-foreground-tertiary">{{ layout.fileSubpath }}</p>
      </div>
    </template>
  </article>
</template>
