<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";

import type { TokenUsageReport, UsagePurpose } from "../../../foundation/ai/token-usage";

const props = defineProps<{ report: TokenUsageReport }>();
const { n, t } = useI18n();
const purposeLabelKeys: Record<UsagePurpose, string> = {
  assistant: "settings.usage.purpose.assistant",
  editor: "settings.usage.purpose.editor",
  knowledge: "settings.usage.purpose.knowledge",
  knowledge_indexing: "settings.usage.purpose.knowledgeIndexing",
  translation: "settings.usage.purpose.translation",
  visual_analysis: "settings.usage.purpose.visualAnalysis",
};
const rows = computed(() =>
  props.report.byPurpose.map((item) =>
    Object.assign({}, item, {
      label: t(purposeLabelKeys[item.purpose]),
      share: props.report.summary.totalTokens > 0 ? item.totalTokens / props.report.summary.totalTokens : 0,
    }),
  ),
);
</script>

<template>
  <section class="rounded-section border border-border bg-panel p-4 shadow-panel">
    <h2 class="text-title-sm font-semibold text-foreground">{{ t("settings.usage.purpose.title") }}</h2>
    <div v-if="rows.length" class="mt-3 grid gap-3">
      <div v-for="row in rows" :key="row.purpose">
        <div class="mb-1.5 flex items-center justify-between gap-4 text-label">
          <span class="text-foreground">{{ row.label }}</span>
          <span class="text-foreground-secondary tabular-nums">{{ n(row.totalTokens, "compact") }}</span>
        </div>
        <div class="h-1.5 overflow-hidden rounded-full bg-control">
          <div class="h-full rounded-full bg-accent" :style="{ width: `${Math.max(row.share * 100, 2)}%` }" />
        </div>
      </div>
    </div>
    <p v-else class="mt-2 text-body text-foreground-secondary">{{ t("settings.usage.activity.empty") }}</p>
  </section>
</template>
