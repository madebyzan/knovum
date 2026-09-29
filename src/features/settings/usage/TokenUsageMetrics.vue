<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";

import type { TokenUsageReport } from "../../../foundation/ai/token-usage";
import UiTooltip from "../../../shared/ui/UiTooltip.vue";

const props = defineProps<{ summary: TokenUsageReport["summary"] }>();
const { n, t } = useI18n();
const splitTotal = computed(() => props.summary.inputTokens + props.summary.outputTokens);
const inputShare = computed(() =>
  splitTotal.value ? (props.summary.inputTokens / splitTotal.value) * 100 : 0,
);
</script>

<template>
  <section class="rounded-xl bg-surface-panel px-5 py-5 sm:px-6 sm:py-6">
    <div class="flex items-start justify-between gap-6">
      <div>
        <h2>
          <UiTooltip
            :text="t('settings.usage.providerReported')"
            side="top"
            class="max-w-72 whitespace-normal"
          >
            <button
              type="button"
              class="cursor-help rounded-sm text-label font-medium text-fg-secondary underline decoration-dotted underline-offset-4 focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-stroke-focus"
            >
              {{ t("settings.usage.tokens.total") }}
            </button>
          </UiTooltip>
        </h2>
        <p class="mt-2 text-display font-semibold tracking-title text-fg-primary tabular-nums">
          {{ n(summary.totalTokens, "compact") }}
        </p>
      </div>
      <div class="text-right">
        <p class="text-caption text-fg-secondary">{{ t("settings.usage.requests") }}</p>
        <p class="mt-2 text-title font-semibold text-fg-primary tabular-nums">
          {{ n(summary.requests) }}
        </p>
      </div>
    </div>
    <p class="mt-6 text-caption text-fg-tertiary">{{ t("settings.usage.tokens.split") }}</p>
    <div class="mt-2 flex h-1.5 overflow-hidden rounded-full bg-surface-control" aria-hidden="true">
      <div class="h-full bg-surface-accent" :style="{ width: `${inputShare}%` }" />
      <div v-if="splitTotal" class="h-full flex-1 bg-surface-accent/40" />
    </div>
    <dl class="mt-4 flex flex-wrap gap-x-8 gap-y-3">
      <div class="flex items-center gap-2">
        <dt class="flex items-center gap-2 text-caption text-fg-secondary">
          <span class="size-2 rounded-full bg-surface-accent" aria-hidden="true" />
          {{ t("settings.usage.tokens.input") }}
        </dt>
        <dd class="text-label font-medium text-fg-primary tabular-nums">
          {{ n(summary.inputTokens, "compact") }}
        </dd>
      </div>
      <div class="flex items-center gap-2">
        <dt class="flex items-center gap-2 text-caption text-fg-secondary">
          <span class="size-2 rounded-full bg-surface-accent/40" aria-hidden="true" />
          {{ t("settings.usage.tokens.output") }}
        </dt>
        <dd class="text-label font-medium text-fg-primary tabular-nums">
          {{ n(summary.outputTokens, "compact") }}
        </dd>
      </div>
    </dl>
  </section>
</template>
