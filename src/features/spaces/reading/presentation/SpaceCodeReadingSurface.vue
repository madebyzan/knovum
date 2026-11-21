<script setup lang="ts">
import { Check, Code2, Copy, Eye, Search, WrapText } from "@lucide/vue";
import { computed, nextTick, shallowRef, useTemplateRef, watch } from "vue";
import { useI18n } from "vue-i18n";

import UiFindBar from "../../../../shared/ui/UiFindBar.vue";
import UiIconButton from "../../../../shared/ui/UiIconButton.vue";
import UiScrollEdgeFade from "../../../../shared/ui/UiScrollEdgeFade.vue";
import UiSegmentedControl from "../../../../shared/ui/UiSegmentedControl.vue";
import type { KnowledgeLocator } from "../../knowledge/model/schema/chunk";
import type { KnowledgeChunk } from "../../knowledge/model/schema/persisted-chunk";
import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import type { SpaceCodePayload } from "../../model/nodes/node-payload";
import { useSpaceStore } from "../../session/space-store";
import { resolveCodePreviewKind } from "../code-preview";
import SpaceCodeViewport from "./SpaceCodeViewport.vue";
import SpaceReaderWebviewHost from "./SpaceReaderWebviewHost.vue";
import { useCodeReader } from "./use-code-reader";
import { useReaderTheme } from "./use-reader-theme";

const props = defineProps<{
  node: SpaceCanvasNode<SpaceCodePayload>;
  chunks: readonly KnowledgeChunk[];
  locator?: KnowledgeLocator | undefined;
}>();

const { t } = useI18n();
const store = useSpaceStore();
const { isDark } = useReaderTheme();
const viewport = useTemplateRef<InstanceType<typeof SpaceCodeViewport>>("viewport");
const wrapLines = shallowRef(true);
const copied = shallowRef(false);

const previewKind = computed(() =>
  resolveCodePreviewKind(props.node.data.previewLanguage, props.node.data.fileName),
);
const canPreview = computed(() =>
  Boolean(previewKind.value && store.activeSpaceId && props.node.data.assetPath),
);
const viewMode = shallowRef<"code" | "preview">(
  canPreview.value && props.locator?.location.kind !== "code" ? "preview" : "code",
);
const codeMounted = shallowRef(viewMode.value === "code");
const reader = useCodeReader({
  node: computed(() => props.node),
  chunks: computed(() => props.chunks),
  isDark,
  viewMode,
});
const modeOptions = computed(() => [
  { value: "preview" as const, label: t("spaces.inspector.reader.preview"), icon: Eye },
  { value: "code" as const, label: t("spaces.nodeTypes.code"), icon: Code2 },
]);
const highlightedRange = computed(() =>
  props.locator?.location.kind === "code"
    ? { start: props.locator.location.lineStart, end: props.locator.location.lineEnd }
    : undefined,
);

function stepMatch(delta: 1 | -1) {
  const total = reader.matchedLineNumbers.value.length;
  if (!total) return;
  reader.matchCursor.value = (reader.matchCursor.value + delta + total) % total;
  const line = reader.matchedLineNumbers.value[reader.matchCursor.value];
  if (line !== undefined) viewport.value?.scrollToLine(line);
}
function openCodeFind() {
  viewMode.value = "code";
  reader.findOpen.value = true;
}
function toggleFind() {
  if (reader.findOpen.value) reader.resetFind();
  else openCodeFind();
}
function onKeyDown(event: KeyboardEvent) {
  if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === "f") {
    event.preventDefault();
    openCodeFind();
  }
}
async function copyAll() {
  const text = await reader.ensureLoaded();
  if (!text) return;
  await navigator.clipboard.writeText(text);
  copied.value = true;
  globalThis.setTimeout(() => (copied.value = false), 1500);
}

watch(
  () => [props.node.id, canPreview.value, props.locator?.location.kind] as const,
  ([, previewable, locKind]) => {
    viewMode.value = previewable && locKind !== "code" ? "preview" : "code";
    codeMounted.value = viewMode.value === "code";
    reader.resetFind();
  },
);
watch([viewMode, reader.activeMatchLine, () => highlightedRange.value?.start], ([mode, match, start]) => {
  if (mode === "code") codeMounted.value = true;
  const targetLine = match ?? (mode === "code" ? start : undefined);
  if (targetLine !== undefined) void nextTick(() => viewport.value?.scrollToLine(targetLine));
});
</script>

<template>
  <div class="flex h-full min-h-0 flex-col bg-surface-canvas outline-none" tabindex="-1" @keydown="onKeyDown">
    <div
      data-reader-ignore-selection
      class="flex h-8.5 shrink-0 items-center justify-between gap-2 border-b border-stroke-divider/60 bg-surface-raised px-3 select-none"
    >
      <div class="flex min-w-0 items-center gap-2 text-caption-sm text-fg-tertiary">
        <UiSegmentedControl v-if="canPreview" v-model="viewMode" :options="modeOptions" />
        <span class="shrink-0 font-mono font-medium uppercase">{{
          reader.language.value || node.data.previewLanguage
        }}</span>
        <span v-if="viewMode === 'code'" class="truncate tabular-nums"
          >{{ reader.rawLines.value.length }} lines</span
        >
      </div>
      <div class="flex shrink-0 items-center gap-0.5">
        <UiIconButton
          v-if="viewMode === 'code'"
          :label="t('spaces.inspector.reader.searchPlaceholder')"
          size="sm"
          tooltip
          :class="reader.findOpen.value && 'bg-surface-control text-fg-primary'"
          @click="toggleFind"
        >
          <Search class="size-3.5" :stroke-width="1.6" />
        </UiIconButton>
        <UiIconButton
          v-if="viewMode === 'code'"
          :label="t('spaces.inspector.reader.wrapLines')"
          size="sm"
          tooltip
          :class="wrapLines && 'bg-surface-control text-fg-primary'"
          @click="wrapLines = !wrapLines"
        >
          <WrapText class="size-3.5" :stroke-width="1.6" />
        </UiIconButton>
        <UiIconButton
          :label="copied ? t('common.copied') : t('common.copy')"
          size="sm"
          tooltip
          @click="copyAll"
        >
          <Check v-if="copied" class="size-3.5 text-fg-accent" :stroke-width="1.75" />
          <Copy v-else class="size-3.5" :stroke-width="1.6" />
        </UiIconButton>
      </div>
    </div>
    <UiFindBar
      v-if="viewMode === 'code' && reader.findOpen.value"
      v-model="reader.findQuery.value"
      :match-current="reader.matchedLineNumbers.value.length ? reader.matchCursor.value + 1 : 0"
      :match-total="reader.matchedLineNumbers.value.length"
      @next="stepMatch(1)"
      @prev="stepMatch(-1)"
      @close="reader.resetFind()"
    />
    <div class="relative min-h-0 flex-1 overflow-hidden">
      <SpaceReaderWebviewHost
        v-if="canPreview && store.activeSpaceId"
        class="absolute inset-0"
        :class="viewMode === 'preview' ? 'visible' : 'pointer-events-none invisible'"
        :inert="viewMode !== 'preview'"
        :active="viewMode === 'preview'"
        :asset="{ spaceId: store.activeSpaceId, nodeId: node.id, revision: node.data.sourceRevision }"
        :show-address-bar="false"
      />
      <SpaceCodeViewport
        v-if="codeMounted"
        ref="viewport"
        class="absolute inset-0"
        :class="viewMode === 'code' ? 'visible' : 'pointer-events-none invisible'"
        :inert="viewMode !== 'code'"
        :raw-lines="reader.rawLines.value"
        :highlighted-lines="reader.highlightedLines.value"
        :wrap-lines="wrapLines"
        :active-match-line="reader.activeMatchLine.value"
        :highlighted-range="highlightedRange"
      />
      <UiScrollEdgeFade v-if="viewMode === 'code'" surface="raised" top-class="h-4" bottom-class="h-8" />
    </div>
  </div>
</template>
