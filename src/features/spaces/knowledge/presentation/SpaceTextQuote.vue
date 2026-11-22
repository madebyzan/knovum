<script setup lang="ts">
import { computed } from "vue";

import { findNormalizedTextRange } from "../model/quote-match";

const props = defineProps<{ text: string; exact?: string | undefined }>();
const hasExact = computed(() => Boolean(props.exact?.trim()));

const parts = computed(() => {
  const range = props.exact ? findNormalizedTextRange(props.text, props.exact) : null;
  if (!range) return [{ text: props.text, match: false }];
  return [
    { text: props.text.slice(0, range.start), match: false },
    { text: props.text.slice(range.start, range.end), match: true },
    { text: props.text.slice(range.end), match: false },
  ].filter((part) => part.text);
});
</script>

<template>
  <p class="whitespace-pre-wrap text-fg-primary">
    <template v-for="(part, index) in parts" :key="index">
      <mark
        v-if="part.match"
        data-citation-mark
        class="rounded-xs bg-surface-selected px-0.5 font-medium text-fg-primary ring-1 ring-stroke-accent/25"
        >{{ part.text }}</mark
      >
      <span v-else :class="hasExact ? 'text-fg-secondary' : 'text-fg-primary'">{{ part.text }}</span>
    </template>
  </p>
</template>
