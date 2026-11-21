<script setup lang="ts">
import { Search } from "@lucide/vue";
import { convertFileSrc } from "@tauri-apps/api/core";
import { computed, nextTick, useTemplateRef, watch } from "vue";
import { useI18n } from "vue-i18n";

import UiFindBar from "../../../../shared/ui/UiFindBar.vue";
import UiIconButton from "../../../../shared/ui/UiIconButton.vue";
import type { KnowledgeLocator } from "../../knowledge/model/schema/chunk";
import type { KnowledgeChunk } from "../../knowledge/model/schema/persisted-chunk";
import { knowledgeLocationLabel } from "../../knowledge/presentation/knowledge-location-label";
import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import { projectDocumentBlocks, type DocumentRenderBlock } from "../document-block-projection";
import SpaceDocumentBlock from "./SpaceDocumentBlock.vue";
import SpaceReaderFontControls from "./SpaceReaderFontControls.vue";
import SpaceReaderOutlineBar, { type ReaderOutlineAnchor } from "./SpaceReaderOutlineBar.vue";
import { useDomBlockFind } from "./use-dom-block-find";
import { useReaderTypography } from "./use-reader-typography";

const props = defineProps<{
  node: SpaceCanvasNode;
  chunks: readonly KnowledgeChunk[];
  loading: boolean;
  locator?: KnowledgeLocator | undefined;
  quote?: string | undefined;
}>();

const { t } = useI18n();
const typography = useReaderTypography();
const scroller = useTemplateRef<HTMLDivElement>("scroller");
const find = useDomBlockFind(scroller, { selector: "[data-chunk-index]" });
const blocks = computed(() => projectDocumentBlocks(props.chunks, props.node.data));
const figures = computed(() =>
  props.node.data.kind === "file"
    ? (props.node.data.documentCoverage?.resources ?? []).map((res) => ({
        id: res.id,
        src: convertFileSrc(res.path),
      }))
    : [],
);

function anchorLabel(block: DocumentRenderBlock): string | undefined {
  if (block.slideNumber !== undefined) return `#${block.slideNumber}`;
  return block.kind === "heading" ? block.text : block.sectionLabel;
}

const outlineAnchors = computed<readonly ReaderOutlineAnchor[]>(() => {
  const seen = new Set<string>();
  const anchors: ReaderOutlineAnchor[] = [];
  for (const block of blocks.value) {
    const label = anchorLabel(block);
    if (!label || seen.has(label)) continue;
    seen.add(label);
    anchors.push({ key: block.chunkIndex, label });
  }
  return anchors.length > 1 ? anchors : [];
});

const activeChunkIndex = computed(() => {
  if (props.locator) {
    const targetLabel = knowledgeLocationLabel(props.locator);
    const byLoc = blocks.value.find(
      (b) => knowledgeLocationLabel({ sourceRevision: "current", location: b.location }) === targetLabel,
    );
    if (byLoc) return byLoc.chunkIndex;
  }
  const needle = props.quote?.trim();
  return needle ? (blocks.value.find((b) => b.text.includes(needle))?.chunkIndex ?? -1) : -1;
});

function scrollToChunk(chunkIndex: string | number) {
  scroller.value
    ?.querySelector<HTMLElement>(`[data-chunk-index="${chunkIndex}"]`)
    ?.scrollIntoView({ behavior: "smooth", block: "center" });
}

watch(
  () => props.node.id,
  () => find.reset(),
);
watch(
  () => [props.locator, props.quote, blocks.value.length] as const,
  async () => {
    if (activeChunkIndex.value < 0) return;
    await nextTick();
    scrollToChunk(activeChunkIndex.value);
  },
  { immediate: true },
);
</script>

<template>
  <div
    class="flex h-full min-h-0 flex-col bg-surface-canvas outline-none"
    tabindex="-1"
    @keydown="find.onKeyDown"
  >
    <div
      data-reader-ignore-selection
      class="flex h-9 shrink-0 items-center justify-between gap-2 border-b border-stroke-divider bg-surface-panel px-2.5 select-none"
    >
      <SpaceReaderOutlineBar :anchors="outlineAnchors" @select="scrollToChunk" />
      <div class="flex shrink-0 items-center gap-1.5">
        <UiIconButton
          :label="t('spaces.inspector.reader.searchPlaceholder')"
          size="sm"
          tooltip
          :class="find.open.value && 'bg-surface-control text-fg-primary'"
          @click="find.toggle()"
        >
          <Search class="size-3.5" />
        </UiIconButton>
        <SpaceReaderFontControls
          :can-decrease="typography.canDecrease.value"
          :can-increase="typography.canIncrease.value"
          @decrease="typography.decrease()"
          @increase="typography.increase()"
        />
      </div>
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

    <div ref="scroller" class="min-h-0 flex-1 overflow-y-auto bg-surface-raised px-7 py-6">
      <div
        v-if="blocks.length"
        class="reader-prose mx-auto max-w-[68ch] space-y-4"
        :style="typography.proseStyle.value"
      >
        <SpaceDocumentBlock
          v-for="(block, idx) in blocks"
          :key="block.id"
          :block="block"
          :show-slide-header="
            block.slideNumber !== undefined &&
            (idx === 0 || blocks[idx - 1]?.slideNumber !== block.slideNumber)
          "
          :show-section-label="
            Boolean(
              block.sectionLabel &&
              block.kind !== 'heading' &&
              (idx === 0 || blocks[idx - 1]?.sectionLabel !== block.sectionLabel),
            )
          "
          :active="block.chunkIndex === activeChunkIndex"
          :exact-quote="block.chunkIndex === activeChunkIndex ? quote : undefined"
        />
        <div
          v-if="figures.length"
          data-reader-ignore-selection
          class="grid grid-cols-2 gap-3 border-t border-stroke-divider pt-4"
        >
          <img
            v-for="figure in figures"
            :key="figure.id"
            :src="figure.src"
            alt=""
            class="max-h-48 w-full rounded-lg border border-stroke-divider bg-surface-control object-contain"
            loading="lazy"
          />
        </div>
      </div>
      <p
        v-else-if="!loading"
        data-reader-ignore-selection
        class="p-6 text-center text-caption text-fg-tertiary select-none"
      >
        {{ t("spaces.conversation.sourcePeek.noText") }}
      </p>
    </div>
  </div>
</template>

<style src="./reader-prose.css"></style>
