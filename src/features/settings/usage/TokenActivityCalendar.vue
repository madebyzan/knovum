<script setup lang="ts">
import { getLocalTimeZone, parseDate } from "@internationalized/date";
import { computed } from "vue";
import { useI18n } from "vue-i18n";

import type { TokenUsageReport } from "../../../foundation/ai/token-usage";
import { buildTokenActivityWeeks } from "./token-activity-calendar";

const props = defineProps<{ report: TokenUsageReport }>();
const { d, locale, n, t } = useI18n();
const weeks = computed(() =>
  buildTokenActivityWeeks(props.report.activity, props.report.fromDate, props.report.toDate, locale.value),
);
const levelClasses = [
  "bg-surface-control",
  "bg-surface-accent/20",
  "bg-surface-accent/40",
  "bg-surface-accent/65",
  "bg-surface-accent",
] as const;
const timeZone = getLocalTimeZone();

function label(date: string, tokens: number, requests: number) {
  const formatted = d(parseDate(date).toDate(timeZone), "medium");
  return t("settings.usage.activity.label", { date: formatted, tokens: n(tokens), requests: n(requests) });
}
</script>

<template>
  <section
    data-testid="token-activity-calendar"
    class="rounded-xl border border-stroke-default bg-surface-panel p-4 shadow-panel"
  >
    <header class="mb-4 flex items-end justify-between gap-4">
      <div>
        <h2 class="text-title-sm font-semibold text-fg-primary">
          {{ t("settings.usage.activity.title") }}
        </h2>
        <p class="mt-1 text-caption-md text-fg-secondary">
          {{
            t("settings.usage.activity.description", { peak: n(report.summary.peakDailyTokens, "compact") })
          }}
        </p>
      </div>
      <div class="flex items-center gap-1.5 text-caption-sm text-fg-tertiary" aria-hidden="true">
        <span>{{ t("settings.usage.activity.less") }}</span>
        <span v-for="level in 5" :key="level" class="size-2.5 rounded-sm" :class="levelClasses[level - 1]" />
        <span>{{ t("settings.usage.activity.more") }}</span>
      </div>
    </header>
    <div class="overflow-x-auto pb-1">
      <div class="min-w-max">
        <div class="mb-1.5 flex gap-1 pl-7 text-caption-sm text-fg-tertiary" aria-hidden="true">
          <div v-for="week in weeks" :key="week.key" class="w-2.5 shrink-0">{{ week.month }}</div>
        </div>
        <div class="flex gap-2">
          <div class="grid grid-rows-7 gap-1 text-caption-sm text-fg-tertiary" aria-hidden="true">
            <span /><span>{{ t("settings.usage.activity.weekdays.monday") }}</span
            ><span /><span>{{ t("settings.usage.activity.weekdays.wednesday") }}</span
            ><span /><span>{{ t("settings.usage.activity.weekdays.friday") }}</span
            ><span />
          </div>
          <div class="flex gap-1" role="grid" :aria-label="t('settings.usage.activity.title')">
            <div v-for="week in weeks" :key="week.key" class="grid grid-rows-7 gap-1" role="row">
              <span
                v-for="cell in week.cells"
                :key="cell.date"
                class="size-2.5 rounded-sm ring-1 ring-stroke-default ring-inset"
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
