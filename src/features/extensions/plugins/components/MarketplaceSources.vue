<script setup lang="ts">
import { useQuery } from "@tanstack/vue-query";
import { useI18n } from "vue-i18n";

import { getErrorMessage } from "../../../../foundation/errors/error-format";
import { marketplaceQuery } from "../../../../foundation/extensions/extension-query";
import {
  addPluginMarketplace,
  refreshPluginMarketplace,
  removePluginMarketplace,
} from "../../../../foundation/extensions/plugin-operations";
import type { MarketplaceSource } from "../../../../shared/contracts/generated/MarketplaceSource";
import { confirmAction } from "../../../../shared/ui/confirm-action";
import UiButton from "../../../../shared/ui/UiButton.vue";
import UiDialog from "../../../../shared/ui/UiDialog.vue";
import { useExtensionChanges } from "../../use-extension-changes";
import MarketplaceSourceForm from "./MarketplaceSourceForm.vue";
const open = defineModel<boolean>("open", { required: true });
const emit = defineEmits<{ removed: [marketplaceId: string] }>();
const { t } = useI18n();
const query = useQuery(marketplaceQuery);
const changes = useExtensionChanges();
function mutate(operation: () => Promise<unknown>) {
  void changes
    .mutateAsync(async () => {
      await operation();
    })
    .catch(() => {});
}
function add(source: MarketplaceSource) {
  mutate(() => addPluginMarketplace(source));
}
async function remove(marketplaceId: string) {
  if (
    await confirmAction({
      title: t("extensions.plugins.catalogs.remove.title"),
      description: t("extensions.plugins.catalogs.remove.description"),
      confirmLabel: t("common.remove"),
      danger: true,
    })
  ) {
    emit("removed", marketplaceId);
    mutate(() => removePluginMarketplace(marketplaceId));
  }
}
</script>
<template>
  <UiDialog
    :open="open"
    @update:open="!changes.isPending.value && (open = $event)"
    :title="$t('extensions.plugins.catalogs.title')"
    :description="$t('extensions.plugins.catalogs.description')"
    :closable="!changes.isPending.value"
  >
    <div class="space-y-5">
      <div v-if="query.data.value?.length" class="divide-y divide-stroke-divider">
        <div
          v-for="source in query.data.value?.filter((marketplace) => marketplace.source.type !== 'bundled')"
          :key="source.id"
          class="flex flex-wrap items-center gap-2 py-3"
        >
          <div class="min-w-0 flex-1">
            <h3 class="text-body font-medium">{{ source.displayName }}</h3>
            <p class="mt-1 text-caption break-all text-fg-tertiary">{{ source.id }}</p>
          </div>
          <UiButton
            size="sm"
            variant="ghost"
            :disabled="changes.isPending.value"
            @click="mutate(() => refreshPluginMarketplace(source.id))"
            >{{ $t("common.refresh") }}</UiButton
          >
          <UiButton
            size="sm"
            variant="ghost"
            :disabled="changes.isPending.value"
            class="text-fg-danger"
            @click="remove(source.id)"
            >{{ $t("common.remove") }}</UiButton
          >
        </div>
      </div>
      <section>
        <h3 class="mb-3 text-body font-medium">{{ $t("extensions.plugins.catalogs.add") }}</h3>
        <MarketplaceSourceForm :busy="changes.isPending.value" @add="add" />
      </section>
      <p v-if="changes.error.value" role="alert" class="text-body text-fg-danger">
        {{ getErrorMessage(changes.error.value) }}
      </p>
    </div>
    <template #footer
      ><UiButton :disabled="changes.isPending.value" data-testid="close-marketplaces" @click="open = false">{{
        $t("common.close")
      }}</UiButton></template
    >
  </UiDialog>
</template>
