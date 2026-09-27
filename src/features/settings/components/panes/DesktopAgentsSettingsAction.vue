<script setup lang="ts">
import { RefreshCw } from "@lucide/vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";

import { useExecutionCatalogQuery } from "../../../../foundation/ai/execution-catalog-query";
import UiButton from "../../../../shared/ui/UiButton.vue";

const { t } = useI18n();
const catalog = useExecutionCatalogQuery();

async function refresh() {
  const installationRefresh = await catalog.refreshInstallations();
  if (installationRefresh.error) toast.error(String(installationRefresh.error));
}
</script>

<template>
  <UiButton size="sm" variant="ghost" :disabled="catalog.installationFetching.value" @click="refresh">
    <RefreshCw class="size-3.5" :class="catalog.installationFetching.value && 'animate-spin'" />
    {{ t("settings.agents.check") }}
  </UiButton>
</template>
