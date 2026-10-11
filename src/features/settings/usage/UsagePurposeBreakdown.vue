<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";

import type { UsagePurpose } from "../../../shared/contracts/generated/UsagePurpose";
import type { UsageReport } from "../../../shared/contracts/generated/UsageReport";

const props = defineProps<{ report: UsageReport }>();
const { n, t } = useI18n();
const PURPOSE_LABEL_KEYS: Record<UsagePurpose, string> = {
  builtin_agent: "settings.usage.purpose.builtinAgent",
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
    label: t(PURPOSE_LABEL_KEYS[item.purpose]),
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
        <span class="shrink-0 text-label font-medium text-fg-primary tabular-nums">
          {{ n(row.totalTokens, "compact") }} Token
        </span>
      </div>
    </div>
    <p v-else class="mt-4 border-t border-stroke-divider pt-4 text-body text-fg-secondary">
      {{ t("settings.usage.activity.empty") }}
    </p>
  </section>
</template>
