<script setup lang="ts">
import type { StateInline, Token } from "markdown-it";
import { computed } from "vue";

import { createMarkdownRenderer, sanitizeRenderedMarkdown } from "../../../../shared/ui/markdown-renderer";
import { findGroundedCitation, type GroundedCitation } from "../../knowledge/model/grounded-citation";
import { knowledgeLocationLabel } from "../../knowledge/model/location";

function escapeAttr(value: string): string {
  return value.replace(/[&<>"]/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;" })[c]!);
}

const props = withDefaults(
  defineProps<{
    content: string;
    citations?: readonly GroundedCitation[];
    activeCitationIndex?: number;
  }>(),
  { citations: () => [], activeCitationIndex: 0 },
);
const emit = defineEmits<{ highlightCitation: [index: number] }>();

const md = createMarkdownRenderer();

md.inline.ruler.before("escape", "citation", (state: StateInline, silent: boolean) => {
  const src = state.src.slice(state.pos);
  const match = src.match(/^\[C(\d+)\]/);
  if (!match) return false;
  const indexText = match[1];
  if (!indexText) return false;
  if (!silent) {
    const token = state.push("citation", "", 0);
    token.content = indexText;
    token.meta = { index: Number.parseInt(indexText, 10) };
  }
  state.pos += match[0].length;
  return true;
});

md.renderer.rules.citation = (tokens: Token[], idx: number) => {
  const token = tokens[idx];
  if (!token?.meta) return "";
  const index = token.meta.index as number;
  const citation = findGroundedCitation(props.citations, index);
  const title = citation
    ? `${citation.nodeTitle} · ${knowledgeLocationLabel(citation.locator)}`
    : `Source ${index}`;
  const activeClass = props.activeCitationIndex === index ? " is-active" : "";
  return `<sup class="citation-badge${activeClass}" data-citation-index="${index}" role="button" tabindex="0" title="${escapeAttr(title)}">${index}</sup>`;
};

const rendered = computed(() => {
  const html = md.render(props.content || "");
  return props.citations.length
    ? sanitizeRenderedMarkdown(html)
    : sanitizeRenderedMarkdown(html.replace(/<sup class="citation-badge"[^>]*>\d+<\/sup>/g, ""));
});

function onClick(event: MouseEvent) {
  if (!props.citations.length) return;
  const target = (event.target as HTMLElement).closest<HTMLElement>(".citation-badge");
  if (!target) return;
  const index = Number.parseInt(target.dataset.citationIndex ?? "", 10);
  if (Number.isNaN(index)) return;
  event.preventDefault();
  emit("highlightCitation", index);
}

function onKeydown(event: KeyboardEvent) {
  if (!props.citations.length) return;
  if (event.key !== "Enter" && event.key !== " ") return;
  const target = (event.target as HTMLElement).closest<HTMLElement>(".citation-badge");
  if (!target) return;
  event.preventDefault();
  const index = Number.parseInt(target.dataset.citationIndex ?? "", 10);
  if (!Number.isNaN(index)) emit("highlightCitation", index);
}
</script>

<template>
  <div
    class="ui-markdown text-body leading-relaxed select-text"
    @click="onClick"
    @keydown="onKeydown"
    v-html="rendered"
  />
</template>

<style scoped src="../../../../shared/ui/markdown-content.css"></style>
<style scoped src="./citation-badge.css"></style>
