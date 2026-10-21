<script setup lang="ts">
import { useTemplateRef, type HTMLAttributes } from "vue";

import { cn } from "./cn";
import type { MarkdownCitations } from "./markdown-renderer";
import { useMarkdownPaint } from "./use-markdown-paint";

const props = defineProps<{
  /** Markdown source; text appended to it renders incrementally, so streamed answers format live. */
  content: string;
  /** Renders `[Cn]` markers whose index is listed as citation buttons. */
  citations?: MarkdownCitations;
  class?: HTMLAttributes["class"];
}>();
const emit = defineEmits<{ cite: [index: number] }>();

const root = useTemplateRef<HTMLDivElement>("root");
useMarkdownPaint(root, () => ({ content: props.content, citations: props.citations }));

function openCitation(event: MouseEvent) {
  const target = event.target instanceof Element ? event.target.closest("[data-citation-index]") : null;
  if (target) emit("cite", Number(target.getAttribute("data-citation-index")));
}
</script>

<template>
  <div
    ref="root"
    :class="cn('ui-markdown text-body leading-relaxed select-text', props.class)"
    @click="openCitation"
  />
</template>

<style scoped src="./markdown-content.css"></style>
