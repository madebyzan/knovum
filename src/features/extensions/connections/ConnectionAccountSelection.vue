<script setup lang="ts">
import { useQuery } from "@tanstack/vue-query";
import { computed } from "vue";
import { useI18n } from "vue-i18n";

import { getErrorMessage } from "../../../foundation/errors/error-format";
import { updateMcpConnection } from "../../../foundation/extensions/connection-operations";
import { accountQuery } from "../../../foundation/extensions/extension-query";
import type { McpConnection } from "../../../shared/contracts/generated/McpConnection";
import UiSelect from "../../../shared/ui/UiSelect.vue";
import { useExtensionChanges } from "../use-extension-changes";

const props = defineProps<{ connection: McpConnection }>();
const { t } = useI18n();
const query = useQuery(accountQuery);
const changes = useExtensionChanges();
const accounts = computed(() =>
  (query.data.value ?? []).filter((account) => account.connectorId === props.connection.connectorId),
);
function bind(id: string) {
  void changes
    .mutateAsync(async () => {
      await updateMcpConnection({ ...props.connection, accountId: id === "none" ? null : id });
    })
    .catch(() => {});
}
const selected = computed({ get: () => props.connection.accountId ?? "none", set: bind });
const options = computed(() => [
  { value: "none", label: t("extensions.connections.account.none") },
  ...accounts.value.map((account) => ({
    value: account.id,
    label: account.displayName,
    disabled: !account.enabled || account.cleanupPending,
  })),
]);
</script>

<template>
  <section class="space-y-1">
    <div class="space-y-1.5">
      <span class="text-caption text-fg-secondary">{{ $t("extensions.connections.account.label") }}</span
      ><UiSelect
        v-model="selected"
        :label="$t('extensions.connections.account.label')"
        :options="options"
        :disabled="query.isPending.value || changes.isPending.value"
        data-testid="select-connector-account"
      />
    </div>
    <p class="text-caption text-fg-secondary">{{ $t("extensions.connections.account.changeHint") }}</p>
    <p v-if="query.error.value || changes.error.value" role="alert" class="text-caption text-fg-danger">
      {{ getErrorMessage(query.error.value || changes.error.value) }}
    </p>
  </section>
</template>
