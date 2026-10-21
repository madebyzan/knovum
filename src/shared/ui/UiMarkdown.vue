<script setup lang="ts">
import { computed, type HTMLAttributes } from "vue";

import { cn } from "./cn";
import { createMarkdownRenderer, renderMarkdown, type MarkdownCitations } from "./markdown-renderer";

const props = defineProps<{
  content: string;
  /** Renders `[Cn]` markers whose index is listed as citation buttons. */
  citations?: MarkdownCitations;
  class?: HTMLAttributes["class"];
}>();
const emit = defineEmits<{ cite: [index: number] }>();

const md = createMarkdownRenderer();

const rendered = computed(() => renderMarkdown(md, props.content || "", props.citations));

function openCitation(event: MouseEvent) {
  const target = event.target instanceof Element ? event.target.closest("[data-citation-index]") : null;
  if (target) emit("cite", Number(target.getAttribute("data-citation-index")));
}
</script>

<template>
  <div
    :class="cn('ui-markdown text-body leading-relaxed select-text', props.class)"
    @click="openCitation"
    v-html="rendered"
  />
</template>

<style scoped src="./markdown-content.css"></style>
