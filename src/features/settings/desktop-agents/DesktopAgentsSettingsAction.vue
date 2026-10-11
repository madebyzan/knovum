<script setup lang="ts">
import { RefreshCw } from "@lucide/vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";

import { useExecutionCatalogQuery } from "../../../foundation/ai/use-execution-catalog-query";
import { getErrorMessage } from "../../../foundation/errors/error-format";
import UiButton from "../../../shared/ui/UiButton.vue";
import UiSettingsHeaderActions from "../../../shared/ui/UiSettingsHeaderActions.vue";

const { t } = useI18n();
const catalog = useExecutionCatalogQuery();

async function refresh() {
  const installationRefresh = await catalog.refreshInstallations();
  if (installationRefresh.error) toast.error(getErrorMessage(installationRefresh.error));
}
</script>

<template>
  <UiSettingsHeaderActions>
    <UiButton size="sm" variant="ghost" :disabled="catalog.installationFetching.value" @click="refresh">
      <RefreshCw class="size-3.5" :class="catalog.installationFetching.value && 'animate-spin'" />
      {{ t("settings.agents.check") }}
    </UiButton>
  </UiSettingsHeaderActions>
</template>
