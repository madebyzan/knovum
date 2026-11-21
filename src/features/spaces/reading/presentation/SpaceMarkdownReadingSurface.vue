<script setup lang="ts">
import { computed, nextTick, useTemplateRef, watch } from "vue";
import { useI18n } from "vue-i18n";

import UiMarkdown from "../../../../shared/ui/UiMarkdown.vue";
import UiOutlineRail, { type UiOutlineItem } from "../../../../shared/ui/UiOutlineRail.vue";
import UiScrollEdgeFade from "../../../../shared/ui/UiScrollEdgeFade.vue";
import type { KnowledgeLocator } from "../../knowledge/model/schema/chunk";
import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import type { SpaceMarkdownPayload, SpaceNotePayload } from "../../model/nodes/node-payload";
import { highlightContainerCodeBlocks } from "../code-highlighter";
import {
  extractMarkdownHeadings,
  extractTiptapHeadings,
  noteReadingHtml,
  scrollProseToLocator,
} from "../note-reading-html";
import SpaceReaderProseToolbar from "./SpaceReaderProseToolbar.vue";
import { useDomBlockFind } from "./use-dom-block-find";
import { useReaderActiveAnchor } from "./use-reader-active-anchor";
import { useReaderTheme } from "./use-reader-theme";
import { useReaderTypography } from "./use-reader-typography";

const props = defineProps<{
  node: SpaceCanvasNode<SpaceMarkdownPayload | SpaceNotePayload>;
  locator?: KnowledgeLocator | undefined;
  quote?: string | undefined;
}>();

const { t } = useI18n();
const { isDark } = useReaderTheme();
const typography = useReaderTypography();
const scrollerRef = useTemplateRef<HTMLDivElement>("scroller");
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
const outlineItems = computed<readonly UiOutlineItem[]>(() => {
  const list =
    props.node.data.kind === "markdown"
      ? extractMarkdownHeadings(props.node.data.text)
      : extractTiptapHeadings(props.node.data.tiptapContent);
  return list.length > 1 ? list.map((h) => ({ id: String(h.index), label: h.text, level: h.level })) : [];
});
const outlineAnchor = useReaderActiveAnchor({
  scroller: scrollerRef,
  selector: "[data-reader-prose] h1, [data-reader-prose] h2, [data-reader-prose] h3, [data-reader-prose] h4",
  keyOf: (_el, index) => String(index),
  validIds: () => outlineItems.value.map((item) => item.id),
  sourceKey: () => [props.node.id, markdownText.value, noteHtml.value],
  enabled: () => outlineItems.value.length > 0,
});

function scrollToHeading(id: string) {
  const el = proseRef.value?.querySelectorAll<HTMLElement>("h1, h2, h3, h4").item(Number(id));
  outlineAnchor.scrollTo(id, el);
}
function scrollToLocator() {
  const heading = props.locator?.location.kind === "markdown" ? props.locator.location.heading : undefined;
  scrollProseToLocator(proseRef.value, heading, props.quote);
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
    <SpaceReaderProseToolbar :find="find" :typography="typography" />
    <div class="relative flex min-h-0 flex-1 overflow-hidden">
      <div ref="scroller" class="min-h-0 flex-1 overflow-y-auto bg-surface-raised pt-7 pr-12 pb-14 pl-8">
        <article class="mx-auto max-w-[66ch]">
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
      <UiScrollEdgeFade surface="raised" />
      <UiOutlineRail
        :items="outlineItems"
        :active-id="outlineAnchor.activeId.value"
        :label="t('spaces.inspector.reader.outline')"
        @select="scrollToHeading"
      />
    </div>
  </div>
</template>

<style src="./reader-prose.css"></style>
