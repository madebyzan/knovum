<script setup lang="ts">
import { computed, type HTMLAttributes } from "vue";

import { cn } from "./cn";
import { createMarkdownRenderer, sanitizeRenderedMarkdown } from "./markdown-renderer";

const props = defineProps<{
  content: string;
  class?: HTMLAttributes["class"];
}>();

const md = createMarkdownRenderer();

const rendered = computed(() => sanitizeRenderedMarkdown(md.render(props.content || "")));
</script>

<template>
  <div :class="cn('ui-markdown text-body-sm leading-relaxed select-text', props.class)" v-html="rendered" />
</template>

<style scoped src="./markdown-content.css"></style>
