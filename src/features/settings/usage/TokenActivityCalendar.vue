<script setup lang="ts">
import { getLocalTimeZone, parseDate } from "@internationalized/date";
import { computed } from "vue";
import { useI18n } from "vue-i18n";

import type { UsageReport } from "../../../shared/contracts/generated/UsageReport";
import UiTooltip from "../../../shared/ui/UiTooltip.vue";
import { buildTokenActivityWeeks } from "./token-activity-calendar";

const props = defineProps<{ report: UsageReport }>();
const { locale, n, t } = useI18n();
const timeZone = getLocalTimeZone();
const dateFormatter = computed(
  () => new Intl.DateTimeFormat(locale.value, { year: "numeric", month: "short", day: "numeric", timeZone }),
);
const weeks = computed(() =>
  buildTokenActivityWeeks({
    activity: props.report.activity,
    fromDate: props.report.fromDate,
    toDate: props.report.toDate,
    locale: locale.value,
  }).map((week) => ({
    key: week.key,
    month: week.month,
    cells: week.cells.map((cell) => ({
      date: cell.date,
      tokens: cell.tokens,
      requests: cell.requests,
      level: cell.level,
      inRange: cell.inRange,
      label: cell.inRange ? label(cell.date, cell.tokens) : undefined,
    })),
  })),
);
const LEVEL_CLASSES = [
  "bg-surface-control",
  "bg-surface-accent/20",
  "bg-surface-accent/40",
  "bg-surface-accent/65",
  "bg-surface-accent",
] as const;

function label(date: string, tokens: number) {
  const formattedDate = dateFormatter.value.format(parseDate(date).toDate(timeZone));
  return `${formattedDate} · ${t("settings.usage.activity.tokenCount", { tokens: n(tokens) })}`;
}
</script>

<template>
  <section data-testid="token-activity-calendar" aria-labelledby="token-activity-title">
    <h2 id="token-activity-title" class="text-title-sm font-semibold text-fg-primary">
      {{ t("settings.usage.activity.title") }}
    </h2>
    <div class="mt-5 ml-7 flex gap-1" aria-hidden="true">
      <span
        v-for="week in weeks"
        :key="week.key"
        class="min-w-0 flex-1 text-caption-sm whitespace-nowrap text-fg-tertiary"
      >
        {{ week.month }}
      </span>
    </div>
    <div class="relative mt-2 ml-7">
      <div
        class="absolute inset-y-0 -left-7 grid w-5 grid-rows-7 gap-1 text-caption-sm text-fg-tertiary"
        aria-hidden="true"
      >
        <span />
        <span class="flex items-center whitespace-nowrap">{{
          t("settings.usage.activity.weekdays.monday")
        }}</span>
        <span />
        <span class="flex items-center whitespace-nowrap">{{
          t("settings.usage.activity.weekdays.wednesday")
        }}</span>
        <span />
        <span class="flex items-center whitespace-nowrap">{{
          t("settings.usage.activity.weekdays.friday")
        }}</span>
        <span />
      </div>
      <div class="flex gap-1" role="group" :aria-label="t('settings.usage.activity.title')">
        <div v-for="week in weeks" :key="week.key" class="flex min-w-0 flex-1 flex-col gap-1">
          <template v-for="cell in week.cells" :key="cell.date">
            <UiTooltip v-if="cell.tokens > 0 && cell.label" :text="cell.label" side="top">
              <span
                class="block aspect-square w-full shrink-0 cursor-help rounded-sm hover:ring-1 hover:ring-stroke-strong"
                :class="LEVEL_CLASSES[cell.level]"
                :aria-label="cell.label"
                role="img"
              />
            </UiTooltip>
            <span
              v-else
              class="block aspect-square w-full shrink-0 rounded-sm"
              :class="cell.inRange ? LEVEL_CLASSES[cell.level] : 'opacity-0'"
              :aria-label="cell.label"
              :aria-hidden="!cell.inRange"
              :role="cell.inRange ? 'img' : undefined"
            />
          </template>
        </div>
      </div>
    </div>
    <div class="mt-4 flex flex-wrap items-center justify-between gap-3 text-caption text-fg-tertiary">
      <p>
        {{ t("settings.usage.activity.description", { peak: n(report.summary.peakDailyTokens, "compact") }) }}
      </p>
      <div class="flex items-center gap-1.5" aria-hidden="true">
        <span>{{ t("settings.usage.activity.less") }}</span>
        <span v-for="level in 5" :key="level" class="size-2.5 rounded-sm" :class="LEVEL_CLASSES[level - 1]" />
        <span>{{ t("settings.usage.activity.more") }}</span>
      </div>
    </div>
  </section>
</template>
