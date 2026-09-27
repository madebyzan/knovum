<script setup lang="ts">
import { computed } from "vue";

import {
  canvasNodePresentationContext,
  type CanvasViewportDetailProfile,
} from "../../../canvas/contract/canvas-detail-profile";
import type { CanvasContentHeightMeasurement } from "../../../canvas/contract/canvas-editor-port";
import { CANVAS_NODE_HEADER_HEIGHT, PREVIEW_CAPTION_HEIGHT } from "../../../canvas/node-size";
import type { CanvasNodeLayoutV1 } from "../../../canvas/schema/document";
import { useImageCommentSession } from "../../../comments/image-comment-session";
import { useImageEditSession } from "../../../image-editing/image-edit-session";
import type { SpaceCanvasNode } from "../../../model/nodes/canvas-node";
import { canvasTextPresentation } from "./canvas-text-appearance";
import { spaceCanvasNodePresentation } from "./space-canvas-node-presentation";
import SpaceCanvasCredential from "./SpaceCanvasCredential.vue";
import SpaceCanvasNodeBody from "./SpaceCanvasNodeBody.vue";
import SpaceCanvasNodeStatus from "./SpaceCanvasNodeStatus.vue";
import SpaceCanvasNote from "./SpaceCanvasNote.vue";
import SpaceCanvasText from "./SpaceCanvasText.vue";
import SpaceCanvasWebsite from "./SpaceCanvasWebsite.vue";

// Canvas text styles are deliberately loaded for this composition surface.
// oxlint-disable-next-line import/no-unassigned-import
import "./canvas-text-content.css";

const props = defineProps<{
  layout: CanvasNodeLayoutV1;
  node: SpaceCanvasNode;
  detailProfile: CanvasViewportDetailProfile;
  interactive: boolean;
  persist: (id: string, data: SpaceCanvasNode["data"]) => Promise<void>;
  fitContentHeight: (id: string, measurement: CanvasContentHeightMeasurement) => void;
}>();
const imageEditSession = useImageEditSession();
const imageComments = useImageCommentSession();
const contentInteractive = computed(() => {
  const active = imageEditSession.active;
  return (
    props.interactive ||
    (props.node.data.kind === "image" &&
      (imageComments.activeNodeId === props.node.id ||
        (active?.nodeId === props.node.id && (active.tool === "eraser" || active.tool === "crop"))))
  );
});
const presentation = computed(() => spaceCanvasNodePresentation(props.node.data));
const isText = computed(() => props.node.data.kind === "text");
const isNote = computed(() => props.node.data.kind === "note");
const hasCredentialAction = computed(() => props.node.data.kind === "credential");
const presentationContext = computed(() =>
  canvasNodePresentationContext(props.detailProfile, props.interactive),
);
const showsSecondaryChrome = computed(() => props.interactive || props.detailProfile.level !== "compact");
const interactiveMedia = computed(
  () => props.interactive && (props.node.data.kind === "audio" || props.node.data.kind === "video"),
);
const visualOnly = computed(() => props.node.data.kind === "image" || props.node.data.kind === "video");
const previewFirst = computed(() => ["audio", "file", "image", "video"].includes(props.node.data.kind));
const textPresentation = computed(() =>
  canvasTextPresentation(isNote.value ? "note" : "text", props.layout.appearance),
);
</script>

<template>
  <article
    class="flex size-full min-h-0 flex-col overflow-hidden text-foreground select-none"
    :class="[
      visualOnly && 'relative',
      isText
        ? 'bg-transparent'
        : isNote
          ? 'rounded-sm shadow-canvas-object'
          : 'rounded-canvas-object border border-border bg-panel shadow-canvas-object',
      (isNote || isText) && textPresentation.classes,
      contentInteractive && !interactiveMedia && !hasCredentialAction && 'pointer-events-auto',
    ]"
    :style="(isNote || isText) && textPresentation.style"
    :data-kind="node.data.kind"
    :aria-hidden="contentInteractive ? undefined : true"
    :inert="contentInteractive ? undefined : true"
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
      :presentation-context="presentationContext"
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
      :presentation-context="presentationContext"
      :persist="persist"
      :vertical-class="textPresentation.verticalClass"
      :fit-content-height="fitContentHeight"
      :font-size="textPresentation.fontSize"
      :frame-width="layout.frame.width"
      :size-authority="layout.sizeAuthority"
      :text-align="textPresentation.textAlign"
      :typography-scale="layout.appearance?.typographyScale ?? 1"
    />
    <SpaceCanvasWebsite
      v-else-if="node.data.kind === 'website'"
      :id="node.id"
      :data="node.data"
      :presentation-context="presentationContext"
    />
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
            v-if="showsSecondaryChrome && presentation.subtitle"
            class="block truncate text-micro text-foreground-tertiary"
            >{{ presentation.subtitle }}</span
          >
        </span>
        <SpaceCanvasNodeStatus :id="node.id" :data="node.data" />
      </header>
      <SpaceCanvasNodeBody
        :id="node.id"
        :data="node.data"
        :presentation-context="presentationContext"
        :interactive="contentInteractive"
      />
      <div
        v-if="visualOnly"
        class="pointer-events-none absolute top-2 right-2 rounded-control bg-panel/85 p-1 shadow-popover empty:hidden"
      >
        <SpaceCanvasNodeStatus :id="node.id" :data="node.data" />
      </div>
      <footer
        v-if="node.data.kind === 'file'"
        class="pointer-events-none flex shrink-0 items-center gap-2 border-t border-divider px-3 py-1.5"
        :style="{ height: `${PREVIEW_CAPTION_HEIGHT}px` }"
      >
        <span class="min-w-0 flex-1">
          <strong class="line-clamp-2 text-body leading-snug font-semibold wrap-break-word">{{
            presentation.title
          }}</strong>
          <span
            v-if="showsSecondaryChrome && presentation.subtitle"
            class="mt-0.5 block truncate text-caption text-foreground-tertiary"
            >{{ presentation.subtitle }}</span
          >
        </span>
        <SpaceCanvasNodeStatus :id="node.id" :data="node.data" />
      </footer>
      <div v-if="layout.fileSubpath && !visualOnly" class="shrink-0 px-3 pb-2">
        <p class="mt-2 truncate text-micro text-foreground-tertiary">{{ layout.fileSubpath }}</p>
      </div>
    </template>
  </article>
</template>
