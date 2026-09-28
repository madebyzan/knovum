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
  props.report.byPurpose.map((item) => ({
    purpose: item.purpose,
    totalTokens: item.totalTokens,
    label: t(purposeLabelKeys[item.purpose]),
  })),
);
</script>

<template>
  <section aria-labelledby="usage-purpose-title">
    <h2 id="usage-purpose-title" class="text-title-sm font-semibold text-fg-primary">
      {{ t("settings.usage.purpose.title") }}
    </h2>
    <div v-if="rows.length" class="mt-4 border-t border-stroke-divider">
      <div v-for="row in rows" :key="row.purpose" class="flex items-center justify-between gap-4 py-3">
        <span class="truncate text-label text-fg-secondary">{{ row.label }}</span>
        <span class="shrink-0 text-label font-medium tabular-nums text-fg-primary">
          {{ n(row.totalTokens, "compact") }} Token
        </span>
      </div>
    </div>
    <p v-else class="mt-4 border-t border-stroke-divider pt-4 text-body text-fg-secondary">
      {{ t("settings.usage.activity.empty") }}
    </p>
  </section>
</template>
