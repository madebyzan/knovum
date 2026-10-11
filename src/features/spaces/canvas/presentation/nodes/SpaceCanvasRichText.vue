<script setup lang="ts">
import { computed } from "vue";

import type { TiptapJsonNode } from "../../../document-editing/tiptap-document";
import { canvasTextHtml } from "./canvas-text-html";

const props = defineProps<{
  content: TiptapJsonNode;
  empty: boolean;
  kind: "note" | "text";
  placeholder: string;
}>();
const html = computed(() => canvasTextHtml(props.content, props.kind));
</script>

<template>
  <div
    class="w-full overflow-hidden wrap-break-word"
    :class="kind === 'note' && 'canvas-text-prose max-h-full leading-relaxed'"
  >
    <span v-if="empty">{{ placeholder }}</span>
    <div v-else class="contents" v-html="html" />
  </div>
</template>
