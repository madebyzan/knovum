<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, useTemplateRef, watch } from "vue";

import { hydrateRichPreviewContainer } from "../../../../../shared/ui/rich-block-hydrator";
import type { TiptapJsonNode } from "../../../document-editing/tiptap-document";
import { canvasTextHtml } from "./canvas-text-html";

const props = defineProps<{
  content: TiptapJsonNode;
  empty: boolean;
  kind: "note" | "text";
  placeholder: string;
}>();
const root = useTemplateRef<HTMLElement>("root");
const html = computed(() => canvasTextHtml(props.content, props.kind));
let themeObserver: MutationObserver | undefined;

async function hydrate() {
  await nextTick();
  if (root.value) void hydrateRichPreviewContainer(root.value);
}

watch([root, html], () => void hydrate(), { immediate: true, flush: "post" });

onMounted(() => {
  if (typeof MutationObserver === "undefined") return;
  themeObserver = new MutationObserver(() => {
    if (root.value?.querySelector("pre > code.language-mermaid")) void hydrate();
  });
  themeObserver.observe(document.documentElement, {
    attributes: true,
    attributeFilter: ["data-theme"],
  });
});

onBeforeUnmount(() => themeObserver?.disconnect());
</script>

<template>
  <div
    ref="root"
    class="w-full overflow-hidden wrap-break-word"
    :class="kind === 'note' && 'canvas-text-prose max-h-full leading-normal'"
  >
    <span v-if="empty">{{ placeholder }}</span>
    <!-- eslint-disable-next-line vue/no-v-html -- sanitized with DOMPurify in canvasTextHtml -->
    <div v-else class="contents" v-html="html" />
  </div>
</template>
