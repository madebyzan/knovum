<script setup lang="ts">
import { computed } from "vue";

const props = defineProps<{ text: string; exact?: string | undefined }>();
const hasExact = computed(() => Boolean(props.exact?.trim()));
const parts = computed(() => {
  const exact = props.exact?.trim();
  if (!exact) return [{ text: props.text, match: false }];
  const index = props.text.indexOf(exact);
  if (index < 0) return [{ text: props.text, match: false }];
  return [
    { text: props.text.slice(0, index), match: false },
    { text: exact, match: true },
    { text: props.text.slice(index + exact.length), match: false },
  ].filter((part) => part.text);
});
</script>

<template>
  <p class="whitespace-pre-wrap text-fg-primary">
    <template v-for="(part, index) in parts" :key="index">
      <mark v-if="part.match" class="rounded-xs bg-surface-selected/90 px-0.5 font-medium text-fg-primary">{{
        part.text
      }}</mark>
      <span v-else :class="hasExact ? 'text-fg-secondary' : 'text-fg-primary'">{{ part.text }}</span>
    </template>
  </p>
</template>
