<script setup lang="ts">
import { computed } from "vue";

const props = defineProps<{ text: string; exact?: string }>();
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
  <p class="text-caption leading-relaxed whitespace-pre-wrap text-foreground-secondary">
    <template v-for="(part, index) in parts" :key="index"
      ><mark v-if="part.match" class="rounded-sm bg-selected px-0.5 text-foreground">{{ part.text }}</mark
      ><template v-else>{{ part.text }}</template></template
    >
  </p>
</template>
