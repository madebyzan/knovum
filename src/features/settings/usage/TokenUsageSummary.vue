<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";

import type { TokenUsageReport } from "../../../foundation/ai/token-usage";

const props = defineProps<{ summary: TokenUsageReport["summary"] }>();
const { n, t } = useI18n();
const items = computed(() => [
  { label: t("settings.usage.tokens.total"), value: n(props.summary.totalTokens, "compact") },
  { label: t("settings.usage.tokens.input"), value: n(props.summary.inputTokens, "compact") },
  { label: t("settings.usage.tokens.output"), value: n(props.summary.outputTokens, "compact") },
  { label: t("settings.usage.requests"), value: n(props.summary.requests) },
  {
    label: t("settings.usage.streak.current"),
    value: t("settings.usage.streak.days", { count: props.summary.currentStreakDays }),
  },
  {
    label: t("settings.usage.streak.longest"),
    value: t("settings.usage.streak.days", { count: props.summary.longestStreakDays }),
  },
]);
</script>

<template>
  <section
    class="grid grid-cols-2 overflow-hidden rounded-section border border-border bg-panel shadow-panel sm:grid-cols-3"
  >
    <div
      v-for="(item, index) in items"
      :key="item.label"
      class="border-divider px-4 py-4 text-center"
      :class="[
        index % 3 !== 2 && 'sm:border-r',
        index < 3 && 'sm:border-b',
        index % 2 === 0 && 'max-sm:border-r',
        index < 4 && 'max-sm:border-b',
      ]"
    >
      <p class="text-title font-semibold tracking-title text-foreground">{{ item.value }}</p>
      <p class="mt-1 text-caption text-foreground-secondary">{{ item.label }}</p>
    </div>
  </section>
</template>
