<script setup lang="ts">
import { useI18n } from "vue-i18n";

import { getErrorMessage } from "../../../foundation/errors/error-format";
import SettingsLoadFailure from "../components/SettingsLoadFailure.vue";
import TokenActivityCalendar from "./TokenActivityCalendar.vue";
import TokenUsageMetrics from "./TokenUsageMetrics.vue";
import UsageActivityInsights from "./UsageActivityInsights.vue";
import UsagePurposeBreakdown from "./UsagePurposeBreakdown.vue";
import { useTokenUsage } from "./use-token-usage";

const { t } = useI18n();
const usage = useTokenUsage();
</script>

<template>
  <SettingsLoadFailure
    v-if="usage.isError.value"
    :message="getErrorMessage(usage.error.value)"
    :can-reset="false"
    @retry="usage.refetch()"
  />
  <div v-else data-testid="usage-settings-pane" class="grid gap-8" :aria-busy="usage.isPending.value">
    <p class="max-w-2xl text-body text-fg-secondary">{{ t("settings.usage.description") }}</p>
    <template v-if="usage.data.value">
      <TokenUsageMetrics :metrics="usage.data.value.metrics" />
      <TokenActivityCalendar :report="usage.data.value" />
      <div class="grid gap-8 lg:grid-cols-2 lg:gap-10">
        <UsageActivityInsights :metrics="usage.data.value.metrics" />
        <UsagePurposeBreakdown :report="usage.data.value" />
      </div>
    </template>
    <div v-else class="h-52 animate-pulse rounded-xl bg-surface-skeleton" />
  </div>
</template>
