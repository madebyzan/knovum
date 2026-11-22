<script setup lang="ts">
import { Maximize2, Search, ZoomIn, ZoomOut } from "@lucide/vue";
import { computed, useTemplateRef, watch } from "vue";
import { useI18n } from "vue-i18n";

import UiFindBar from "../../../../shared/ui/UiFindBar.vue";
import UiIconButton from "../../../../shared/ui/UiIconButton.vue";
import UiScrollEdgeFade from "../../../../shared/ui/UiScrollEdgeFade.vue";
import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import type { SpaceFilePayload } from "../../model/nodes/node-payload";
import { useDocxPreview } from "./use-docx-preview";
import { useDomBlockFind } from "./use-dom-block-find";

const props = defineProps<{
  node: SpaceCanvasNode<SpaceFilePayload>;
  quote?: string | undefined;
  locateNonce?: number | undefined;
}>();

const { t } = useI18n();
const scrollerRef = useTemplateRef<HTMLDivElement>("scroller");
const hostRef = useTemplateRef<HTMLDivElement>("host");
const styleRef = useTemplateRef<HTMLDivElement>("styleHost");

const find = useDomBlockFind(hostRef, {
  selector: "p, td, li, h1, h2, h3, h4",
});
const docx = useDocxPreview({
  scroller: scrollerRef,
  host: hostRef,
  styleHost: styleRef,
  assetPath: () => props.node.data.assetPath,
  active: computed(() => true),
  onRendered: () => {
    if (props.quote?.trim()) find.runFind(props.quote);
  },
});

function onKeyDown(event: KeyboardEvent) {
  if (find.onKeyDown(event) || !(event.metaKey || event.ctrlKey)) return;
  const delta = event.key === "=" || event.key === "+" ? 0.1 : event.key === "-" ? -0.1 : 0;
  if (delta) {
    event.preventDefault();
    docx.adjustZoom(delta);
  } else if (event.key === "0") {
    event.preventDefault();
    docx.resetFit();
  }
}

watch(
  () => props.node.id,
  () => find.reset(),
);
watch(
  () => [props.quote, props.locateNonce] as const,
  ([quote]) => {
    if (!docx.rendering.value && quote?.trim()) find.runFind(quote);
  },
  { flush: "post" },
);
</script>

<template>
  <div class="flex h-full min-h-0 flex-col bg-surface-canvas outline-none" tabindex="-1" @keydown="onKeyDown">
    <div
      data-reader-ignore-selection
      class="flex h-8.5 shrink-0 items-center justify-end gap-0.5 border-b border-stroke-divider/70 bg-surface-canvas px-3 select-none"
    >
      <UiIconButton
        :label="t('spaces.inspector.reader.searchPlaceholder')"
        size="sm"
        tooltip
        :class="find.open.value && 'bg-surface-control text-fg-primary'"
        @click="find.toggle()"
      >
        <Search class="size-3.5" :stroke-width="1.6" />
      </UiIconButton>
      <span class="mx-0.5 h-3.5 w-px bg-stroke-divider" />
      <UiIconButton
        :label="t('spaces.inspector.pdfReader.zoomOut')"
        size="sm"
        tooltip
        @click="docx.adjustZoom(-0.1)"
      >
        <ZoomOut class="size-3.5" :stroke-width="1.6" />
      </UiIconButton>
      <button
        type="button"
        class="min-w-10 rounded-sm px-1 py-0.5 text-center text-caption-sm text-fg-secondary tabular-nums hover:bg-surface-control-hover hover:text-fg-primary"
        @click="docx.resetFit()"
      >
        {{ docx.zoomPercent.value }}%
      </button>
      <UiIconButton
        :label="t('spaces.inspector.pdfReader.zoomIn')"
        size="sm"
        tooltip
        @click="docx.adjustZoom(0.1)"
      >
        <ZoomIn class="size-3.5" :stroke-width="1.6" />
      </UiIconButton>
      <UiIconButton
        :label="t('spaces.inspector.pdfReader.fitWidth')"
        size="sm"
        tooltip
        @click="docx.resetFit()"
      >
        <Maximize2 class="size-3.5" :stroke-width="1.6" />
      </UiIconButton>
    </div>
    <UiFindBar
      v-if="find.open.value"
      v-model="find.query.value"
      :match-current="find.matched.value.length ? find.matchIndex.value + 1 : 0"
      :match-total="find.matched.value.length"
      @next="find.next()"
      @prev="find.prev()"
      @close="find.reset()"
    />
    <div class="relative flex min-h-0 flex-1 overflow-hidden">
      <div ref="scroller" class="min-h-0 flex-1 overflow-auto bg-surface-muted">
        <div ref="styleHost" aria-hidden="true" />
        <p v-if="docx.failure.value" class="p-6 text-center text-caption text-fg-danger" role="alert">
          {{ docx.failure.value }}
        </p>
        <p
          v-else-if="docx.rendering.value"
          data-reader-ignore-selection
          class="p-6 text-center text-caption text-fg-tertiary select-none"
        >
          {{ t("common.loading") }}
        </p>
        <div ref="host" class="word-preview-host px-3 pt-4 pb-10" :style="{ zoom: docx.zoom.value }" />
      </div>
      <UiScrollEdgeFade surface="muted" top-class="h-5" bottom-class="h-9" />
    </div>
  </div>
</template>

<style scoped>
.word-preview-host :deep(.knovum-docx-wrapper) {
  background: transparent;
  padding: 0;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 0.75rem;
}
.word-preview-host :deep(section.knovum-docx) {
  margin-bottom: 0;
  background: #ffffff;
  color: #1a1a1a;
  border: 1px solid rgb(0 0 0 / 0.08);
  border-radius: 2px;
  box-shadow: 0 1px 3px rgb(0 0 0 / 0.08);
}
</style>
