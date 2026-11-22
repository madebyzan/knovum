<script setup lang="ts">
import { convertFileSrc } from "@tauri-apps/api/core";
import { computed, nextTick, onMounted, useTemplateRef, watch } from "vue";
import { useI18n } from "vue-i18n";

import UiOutlineRail, { type UiOutlineItem } from "../../../../shared/ui/UiOutlineRail.vue";
import UiScrollEdgeFade from "../../../../shared/ui/UiScrollEdgeFade.vue";
import { knowledgeLocationLabel } from "../../knowledge/model/location-label";
import type { KnowledgeLocator } from "../../knowledge/model/schema/chunk";
import type { KnowledgeChunk } from "../../knowledge/model/schema/persisted-chunk";
import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import { projectDocumentBlocks, type DocumentRenderBlock } from "../document-block-projection";
import SpaceDocumentBlock from "./SpaceDocumentBlock.vue";
import SpaceReaderProseToolbar from "./SpaceReaderProseToolbar.vue";
import { useDomBlockFind } from "./use-dom-block-find";
import { useReaderActiveAnchor } from "./use-reader-active-anchor";
import { useReaderTypography } from "./use-reader-typography";

const props = defineProps<{
  node: SpaceCanvasNode;
  chunks: readonly KnowledgeChunk[];
  loading: boolean;
  locator?: KnowledgeLocator | undefined;
  locateNonce?: number | undefined;
  quote?: string | undefined;
}>();

const { t } = useI18n();
const typography = useReaderTypography();
const scroller = useTemplateRef<HTMLDivElement>("scroller");
const find = useDomBlockFind(scroller, { selector: "[data-chunk-index]" });
const blocks = computed(() => projectDocumentBlocks(props.chunks, props.node.data));
const figures = computed(() =>
  props.node.data.kind === "file"
    ? (props.node.data.documentCoverage?.resources ?? []).map((r) => ({
        id: r.id,
        src: convertFileSrc(r.path),
      }))
    : [],
);

function anchorLabel(block: DocumentRenderBlock): string | undefined {
  if (block.slideNumber !== undefined) return `#${block.slideNumber}`;
  return block.kind === "heading" ? block.text : block.sectionLabel;
}

const outlineItems = computed<readonly UiOutlineItem[]>(() => {
  const seen = new Set<string>();
  const items: UiOutlineItem[] = [];
  for (const block of blocks.value) {
    const label = anchorLabel(block);
    if (!label || seen.has(label)) continue;
    seen.add(label);
    items.push({ id: String(block.chunkIndex), label, level: block.kind === "heading" ? 1 : 2 });
  }
  return items.length > 1 ? items : [];
});

const outlineAnchor = useReaderActiveAnchor({
  scroller,
  selector: "[data-chunk-index]",
  keyOf: (el) => el.dataset.chunkIndex,
  validIds: () => outlineItems.value.map((item) => item.id),
  sourceKey: () => [props.node.id, blocks.value.length],
  enabled: () => outlineItems.value.length > 0,
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
  const el = scroller.value?.querySelector<HTMLElement>(`[data-chunk-index="${chunkIndex}"]`);
  outlineAnchor.scrollTo(String(chunkIndex), el);
}

async function locateActiveChunk() {
  if (activeChunkIndex.value < 0) return;
  await nextTick();
  const el = scroller.value?.querySelector<HTMLElement>(`[data-chunk-index="${activeChunkIndex.value}"]`);
  const mark = el?.querySelector<HTMLElement>("mark[data-citation-mark]");
  if (mark) {
    mark.scrollIntoView({ behavior: "smooth", block: "center" });
    return;
  }
  scrollToChunk(activeChunkIndex.value);
}

onMounted(() => void locateActiveChunk());
watch(
  () => props.node.id,
  () => find.reset(),
);
watch(
  () => [props.locator, props.quote, props.locateNonce, blocks.value.length] as const,
  () => void locateActiveChunk(),
  { flush: "post" },
);
</script>

<template>
  <div
    class="flex h-full min-h-0 flex-col bg-surface-canvas outline-none"
    tabindex="-1"
    @keydown="find.onKeyDown"
  >
    <SpaceReaderProseToolbar :find="find" :typography="typography" />
    <div class="relative flex min-h-0 flex-1 overflow-hidden">
      <div ref="scroller" class="min-h-0 flex-1 overflow-y-auto bg-surface-raised pt-7 pr-12 pb-14 pl-8">
        <div
          v-if="blocks.length"
          class="reader-prose mx-auto max-w-[66ch] space-y-4"
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
      <UiScrollEdgeFade surface="raised" />
      <UiOutlineRail
        :items="outlineItems"
        :active-id="outlineAnchor.activeId.value"
        :label="t('spaces.inspector.reader.outline')"
        @select="scrollToChunk"
      />
    </div>
  </div>
</template>

<style src="./reader-prose.css"></style>
