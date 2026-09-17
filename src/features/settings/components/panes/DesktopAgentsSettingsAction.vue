<script setup lang="ts">
import { RefreshCw } from "@lucide/vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";

import { useExecutionCatalogQuery } from "../../../../foundation/ai/execution-catalog-query";
import UiButton from "../../../../shared/ui/UiButton.vue";

const { t } = useI18n();
const catalog = useExecutionCatalogQuery();

async function refresh() {
  const result = await catalog.refreshInstallations();
  if (result.error) toast.error(String(result.error));
}
</script>

<template>
  <UiButton size="sm" variant="ghost" :disabled="catalog.installationFetching.value" @click="refresh">
    <RefreshCw
      class="size-(--knovum-icon-size-sm)"
      :class="catalog.installationFetching.value && 'animate-spin'"
    />
    {{ t("settings.ai.checkDesktopAgents") }}
  </UiButton>
</template>
