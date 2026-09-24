<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";

import UiMarkdown from "../../../../../shared/ui/UiMarkdown.vue";
import type { CanvasNodePresentationContext } from "../../../canvas/contract/canvas-detail-profile";
import type { SpaceMarkdownPayload } from "../../../model/nodes/node-payload";
import { nodePayloadWork } from "./canvas-node-work";
import { projectMarkdownPreview } from "./markdown-preview-projection";
import SpaceCanvasProcessingState from "./SpaceCanvasProcessingState.vue";

const props = defineProps<{
  data: SpaceMarkdownPayload;
  presentationContext: CanvasNodePresentationContext;
}>();
const { t } = useI18n();
const work = computed(() => nodePayloadWork(props.data));
const projection = computed(() =>
  props.presentationContext.mode === "interactive"
    ? { projectedSource: props.data.text, remainingBlockCount: 0 }
    : projectMarkdownPreview(props.data.text, props.presentationContext.viewport.previewItemLimit),
);
</script>

<template>
  <div class="relative flex min-h-0 flex-1 overflow-hidden">
    <UiMarkdown
      v-if="work?.surface !== 'content' && data.text"
      :content="projection.projectedSource"
      class="canvas-markdown-preview min-h-0 flex-1 overflow-hidden px-3 py-2.5 text-caption leading-relaxed text-foreground-secondary"
    />
    <p
      v-else-if="work?.surface !== 'content'"
      class="min-h-0 flex-1 px-3 py-2.5 text-caption text-foreground-tertiary"
    >
      {{ t("spaces.editor.emptyMarkdown") }}
    </p>
    <Transition
      leave-active-class="transition-opacity duration-200 motion-reduce:transition-none"
      leave-to-class="opacity-0"
    >
      <SpaceCanvasProcessingState
        v-if="work?.surface === 'content'"
        class="absolute inset-0"
        :label="t(work.labelKey)"
        :phase="work.phase"
      />
    </Transition>
    <span
      v-if="projection.remainingBlockCount > 0"
      class="pointer-events-none absolute right-2 bottom-1 rounded-control bg-panel/90 px-1.5 text-micro text-foreground-tertiary"
    >
      +{{ projection.remainingBlockCount }}
    </span>
  </div>
</template>
