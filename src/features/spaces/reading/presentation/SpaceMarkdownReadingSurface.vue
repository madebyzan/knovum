<script setup lang="ts">
import { Pencil, Search } from "@lucide/vue";
import { computed, nextTick, useTemplateRef, watch } from "vue";
import { useI18n } from "vue-i18n";

import UiButton from "../../../../shared/ui/UiButton.vue";
import UiFindBar from "../../../../shared/ui/UiFindBar.vue";
import UiIconButton from "../../../../shared/ui/UiIconButton.vue";
import UiMarkdown from "../../../../shared/ui/UiMarkdown.vue";
import type { KnowledgeLocator } from "../../knowledge/model/schema/chunk";
import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import type { SpaceMarkdownPayload, SpaceNotePayload } from "../../model/nodes/node-payload";
import { useSpaceStore } from "../../session/space-store";
import { highlightContainerCodeBlocks } from "../code-highlighter";
import { extractMarkdownHeadings, extractTiptapHeadings, noteReadingHtml } from "../note-reading-html";
import SpaceReaderFontControls from "./SpaceReaderFontControls.vue";
import SpaceReaderOutlineBar, { type ReaderOutlineAnchor } from "./SpaceReaderOutlineBar.vue";
import { useDomBlockFind } from "./use-dom-block-find";
import { useReaderTheme } from "./use-reader-theme";
import { useReaderTypography } from "./use-reader-typography";

const props = defineProps<{
  node: SpaceCanvasNode<SpaceMarkdownPayload | SpaceNotePayload>;
  locator?: KnowledgeLocator | undefined;
  quote?: string | undefined;
}>();

const { t } = useI18n();
const store = useSpaceStore();
const { isDark } = useReaderTheme();
const typography = useReaderTypography();
const proseRef = useTemplateRef<HTMLElement>("prose");
const find = useDomBlockFind(proseRef);

const isMarkdown = computed(() => props.node.data.kind === "markdown");
const markdownText = computed(() => (props.node.data.kind === "markdown" ? props.node.data.text : ""));
const noteHtml = computed(() =>
  props.node.data.kind === "note" ? noteReadingHtml(props.node.data.tiptapContent) : "",
);
const isEmpty = computed(() =>
  props.node.data.kind === "markdown" ? !props.node.data.text.trim() : !props.node.data.plainText.trim(),
);
const showInlineTitle = computed(() => {
  const title = props.node.data.title.trim().toLowerCase();
  if (!title) return false;
  const raw = props.node.data.kind === "markdown" ? props.node.data.text : props.node.data.plainText;
  const first = raw.split(/\r?\n/u).find((l) => l.trim().length > 0);
  return (
    first
      ?.replace(/^#+\s*/u, "")
      .trim()
      .toLowerCase() !== title
  );
});
const outlineAnchors = computed<readonly ReaderOutlineAnchor[]>(() => {
  const list =
    props.node.data.kind === "markdown"
      ? extractMarkdownHeadings(props.node.data.text)
      : extractTiptapHeadings(props.node.data.tiptapContent);
  return list.length > 1 ? list.map((h) => ({ key: h.index, label: h.text })) : [];
});

function scrollToHeading(index: string | number) {
  proseRef.value
    ?.querySelectorAll<HTMLElement>("h1, h2, h3, h4")
    .item(Number(index))
    ?.scrollIntoView({ behavior: "smooth", block: "start" });
}
function scrollToLocator() {
  const container = proseRef.value;
  if (!container) return;
  const targetHeading =
    props.locator?.location.kind === "markdown" ? props.locator.location.heading?.trim() : undefined;
  if (targetHeading) {
    const match = [...container.querySelectorAll<HTMLElement>("h1, h2, h3, h4")].find(
      (h) => h.textContent?.trim() === targetHeading,
    );
    if (match) {
      match.scrollIntoView({ behavior: "smooth", block: "start" });
      return;
    }
  }
  const needle = props.quote?.trim();
  if (!needle) return;
  [...container.querySelectorAll<HTMLElement>("p, li, blockquote, pre, h1, h2, h3, h4")]
    .find((el) => el.textContent?.includes(needle))
    ?.scrollIntoView({ behavior: "smooth", block: "center" });
}

watch(
  [proseRef, markdownText, noteHtml, isDark],
  async ([container]) => {
    if (!container) return;
    await nextTick();
    await highlightContainerCodeBlocks(container, isDark.value);
  },
  { immediate: true, flush: "post" },
);
watch(
  () => props.node.id,
  () => find.reset(),
);
watch(
  () => [props.node.id, props.locator, props.quote] as const,
  async () => {
    await nextTick();
    scrollToLocator();
  },
  { immediate: true },
);
</script>

<template>
  <div
    class="flex h-full min-h-0 flex-col bg-surface-raised outline-none"
    tabindex="-1"
    @keydown="find.onKeyDown"
  >
    <div
      data-reader-ignore-selection
      class="flex h-9 shrink-0 items-center justify-between gap-2 border-b border-stroke-divider bg-surface-panel px-2.5 select-none"
    >
      <SpaceReaderOutlineBar :anchors="outlineAnchors" @select="scrollToHeading" />
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
        <UiButton variant="secondary" size="sm" class="gap-1" @click="store.openDocumentEditor(node.id)">
          <Pencil class="size-3" />
          <span>{{ t("common.edit") }}</span>
        </UiButton>
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
    <div class="min-h-0 flex-1 overflow-y-auto bg-surface-raised px-7 py-6">
      <article class="mx-auto max-w-[68ch]">
        <h1
          v-if="showInlineTitle"
          data-reader-ignore-selection
          class="mb-5 text-[1.45rem] leading-tight font-semibold tracking-tight break-words text-fg-primary select-none"
        >
          {{ node.data.title }}
        </h1>
        <div
          v-if="!isEmpty"
          ref="prose"
          data-reader-prose
          class="reader-prose select-text"
          :style="typography.proseStyle.value"
        >
          <UiMarkdown v-if="isMarkdown" :content="markdownText" class="!text-inherit" />
          <!-- eslint-disable-next-line vue/no-v-html -- sanitized with DOMPurify in noteReadingHtml -->
          <div v-else v-html="noteHtml" />
        </div>
        <p
          v-else
          data-reader-ignore-selection
          class="py-10 text-center text-caption text-fg-tertiary select-none"
        >
          {{ t("spaces.conversation.sourcePeek.noText") }}
        </p>
      </article>
    </div>
  </div>
</template>

<style src="./reader-prose.css"></style>
