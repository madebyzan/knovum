<script setup lang="ts">
import { useTemplateRef, type HTMLAttributes } from "vue";

import { linkDestination } from "../contracts/link-destination";
import { cn } from "./cn";
import type { MarkdownCitations } from "./markdown-renderer";
import { useMarkdownPaint } from "./use-markdown-paint";

const props = defineProps<{
  /** Markdown source; text appended to it renders incrementally, so streamed answers format live. */
  content: string;
  /** Renders `[Cn]` markers whose index is listed as citation buttons. */
  citations?: MarkdownCitations;
  /** Keeps links to local files and reports a press on one through `link`; read once, at mount. */
  localLinks?: boolean;
  class?: HTMLAttributes["class"];
}>();
const emit = defineEmits<{ cite: [index: number]; link: [href: string] }>();

const root = useTemplateRef<HTMLDivElement>("root");
useMarkdownPaint(root, () => ({ content: props.content, citations: props.citations }), {
  localLinks: props.localLinks,
});

function press(event: MouseEvent) {
  const target = event.target instanceof Element ? event.target : null;
  const citation = target?.closest("[data-citation-index]");
  if (citation) {
    emit("cite", Number(citation.getAttribute("data-citation-index")));
    return;
  }
  const href = target?.closest("a[href]")?.getAttribute("href");
  if (!props.localLinks || !href || linkDestination(href) !== "local") return;
  event.preventDefault();
  emit("link", href);
}
</script>

<template>
  <div
    ref="root"
    :class="cn('ui-markdown text-body leading-relaxed select-text', props.class)"
    @click="press"
  />
</template>

<style scoped src="./markdown-content.css"></style>
