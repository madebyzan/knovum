<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";

import type { TokenUsageReport } from "../../../foundation/ai/token-usage";
import { buildTokenActivityWeeks } from "./token-activity-calendar";

const props = defineProps<{ report: TokenUsageReport }>();
const { locale, n, t } = useI18n();
const weeks = computed(() =>
  buildTokenActivityWeeks(props.report.activity, props.report.fromDate, props.report.toDate, locale.value),
);
const levelClasses = ["bg-control", "bg-accent/20", "bg-accent/40", "bg-accent/65", "bg-accent"] as const;
const dateFormatter = computed(() => new Intl.DateTimeFormat(locale.value, { dateStyle: "medium" }));

function label(date: string, tokens: number, requests: number) {
  const formatted = dateFormatter.value.format(new Date(`${date}T12:00:00`));
  return t("settings.usage.activityLabel", { date: formatted, tokens: n(tokens), requests: n(requests) });
}
</script>

<template>
  <section
    data-testid="token-activity-calendar"
    class="rounded-section border border-border bg-panel p-4 shadow-panel"
  >
    <header class="mb-4 flex items-end justify-between gap-4">
      <div>
        <h2 class="text-title-sm font-semibold text-foreground">{{ t("settings.usage.activity") }}</h2>
        <p class="mt-1 text-caption text-foreground-secondary">
          {{
            t("settings.usage.activityDescription", { peak: n(report.summary.peakDailyTokens, "compact") })
          }}
        </p>
      </div>
      <div class="flex items-center gap-1.5 text-micro text-foreground-tertiary" aria-hidden="true">
        <span>{{ t("settings.usage.less") }}</span>
        <span
          v-for="level in 5"
          :key="level"
          class="rounded-grid size-2.5"
          :class="levelClasses[level - 1]"
        />
        <span>{{ t("settings.usage.more") }}</span>
      </div>
    </header>
    <div class="overflow-x-auto pb-1">
      <div class="min-w-max">
        <div class="mb-1.5 flex gap-1 pl-7 text-micro text-foreground-tertiary" aria-hidden="true">
          <div v-for="week in weeks" :key="week.key" class="w-2.5 shrink-0">{{ week.month }}</div>
        </div>
        <div class="flex gap-2">
          <div class="grid grid-rows-7 gap-1 text-micro text-foreground-tertiary" aria-hidden="true">
            <span /><span>{{ t("settings.usage.weekdayMonday") }}</span
            ><span /><span>{{ t("settings.usage.weekdayWednesday") }}</span
            ><span /><span>{{ t("settings.usage.weekdayFriday") }}</span
            ><span />
          </div>
          <div class="flex gap-1" role="grid" :aria-label="t('settings.usage.activity')">
            <div v-for="week in weeks" :key="week.key" class="grid grid-rows-7 gap-1" role="row">
              <span
                v-for="cell in week.cells"
                :key="cell.date"
                class="rounded-grid size-2.5 ring-1 ring-border ring-inset"
                :class="cell.inRange ? levelClasses[cell.level] : 'opacity-0'"
                :title="cell.inRange ? label(cell.date, cell.tokens, cell.requests) : undefined"
                :aria-label="cell.inRange ? label(cell.date, cell.tokens, cell.requests) : undefined"
                :aria-hidden="!cell.inRange"
                role="gridcell"
              />
            </div>
          </div>
        </div>
      </div>
    </div>
  </section>
</template>
