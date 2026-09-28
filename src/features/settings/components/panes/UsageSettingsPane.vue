<script setup lang="ts">
import { useI18n } from "vue-i18n";

import TokenActivityCalendar from "../../usage/TokenActivityCalendar.vue";
import TokenUsageMetrics from "../../usage/TokenUsageMetrics.vue";
import UsageActivityInsights from "../../usage/UsageActivityInsights.vue";
import UsagePurposeBreakdown from "../../usage/UsagePurposeBreakdown.vue";
import { useTokenUsage } from "../../usage/use-token-usage";
import SettingsLoadFailure from "../SettingsLoadFailure.vue";

const { t } = useI18n();
const usage = useTokenUsage();
</script>

<template>
  <SettingsLoadFailure
    v-if="usage.isError.value"
    :message="String(usage.error.value)"
    :can-reset="false"
    @retry="usage.refetch()"
  />
  <div v-else data-testid="usage-settings-pane" class="grid gap-8" :aria-busy="usage.isPending.value">
    <p class="max-w-2xl text-body text-fg-secondary">{{ t("settings.usage.description") }}</p>
    <template v-if="usage.data.value">
      <TokenUsageMetrics :summary="usage.data.value.summary" />
      <TokenActivityCalendar :report="usage.data.value" />
      <div class="grid gap-8 lg:grid-cols-2 lg:gap-10">
        <UsageActivityInsights :summary="usage.data.value.summary" />
        <UsagePurposeBreakdown :report="usage.data.value" />
      </div>
    </template>
    <div v-else class="h-52 animate-pulse rounded-xl bg-surface-skeleton" />
  </div>
</template>
