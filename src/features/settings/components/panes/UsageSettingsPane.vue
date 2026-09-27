<script setup lang="ts">
import { shallowRef } from "vue";
import { useI18n } from "vue-i18n";

import UiButton from "../../../../shared/ui/UiButton.vue";
import TokenActivityCalendar from "../../usage/TokenActivityCalendar.vue";
import TokenUsageMetrics from "../../usage/TokenUsageMetrics.vue";
import UsagePurposeBreakdown from "../../usage/UsagePurposeBreakdown.vue";
import { useTokenUsage, type UsageRange } from "../../usage/use-token-usage";
import SettingsLoadFailure from "../SettingsLoadFailure.vue";

const { t } = useI18n();
const range = shallowRef<UsageRange>("year");
const usage = useTokenUsage(range);
</script>

<template>
  <SettingsLoadFailure
    v-if="usage.isError.value"
    :message="String(usage.error.value)"
    :can-reset="false"
    @retry="usage.refetch()"
  />
  <div
    v-else
    data-testid="usage-settings-pane"
    class="grid gap-5"
    :class="usage.isPending.value && 'pointer-events-none opacity-60'"
  >
    <div class="flex items-start justify-between gap-4">
      <p class="max-w-2xl text-body text-fg-secondary">{{ t("settings.usage.description") }}</p>
      <div class="flex shrink-0 rounded-md bg-surface-control p-0.5">
        <UiButton
          v-for="option in ['quarter', 'year'] as const"
          :key="option"
          size="sm"
          :variant="range === option ? 'secondary' : 'ghost'"
          @click="range = option"
        >
          {{ t(`settings.usage.range.${option}`) }}
        </UiButton>
      </div>
    </div>
    <template v-if="usage.data.value">
      <TokenUsageMetrics :summary="usage.data.value.summary" />
      <TokenActivityCalendar :report="usage.data.value" />
      <UsagePurposeBreakdown :report="usage.data.value" />
      <p class="text-caption text-fg-tertiary">{{ t("settings.usage.providerReported") }}</p>
    </template>
    <div v-else class="h-72 animate-pulse rounded-xl bg-surface-skeleton" />
  </div>
</template>
