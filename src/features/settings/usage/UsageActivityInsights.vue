<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";

import type { UsageReport } from "../../../shared/contracts/generated/UsageReport";

const props = defineProps<{ summary: UsageReport["summary"] }>();
const { locale, t } = useI18n();
const dayFormatter = computed(
  () => new Intl.NumberFormat(locale.value, { style: "unit", unit: "day", unitDisplay: "long" }),
);
const rows = computed(() => [
  { label: t("settings.usage.activeDays"), days: props.summary.activeDays },
  { label: t("settings.usage.streak.current"), days: props.summary.currentStreakDays },
  { label: t("settings.usage.streak.longest"), days: props.summary.longestStreakDays },
]);
</script>

<template>
  <section aria-labelledby="usage-activity-insights-title">
    <h2 id="usage-activity-insights-title" class="text-title-sm font-semibold text-fg-primary">
      {{ t("settings.usage.activity.insightsTitle") }}
    </h2>
    <dl class="mt-4 border-t border-stroke-divider">
      <div v-for="row in rows" :key="row.label" class="flex items-center justify-between gap-4 py-3">
        <dt class="text-label text-fg-secondary">{{ row.label }}</dt>
        <dd class="text-label font-medium text-fg-primary tabular-nums">
          {{ dayFormatter.format(row.days) }}
        </dd>
      </div>
    </dl>
  </section>
</template>
