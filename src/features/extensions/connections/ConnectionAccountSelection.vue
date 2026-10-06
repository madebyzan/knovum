<script setup lang="ts">
import { useQuery } from "@tanstack/vue-query";
import { computed } from "vue";

import { getErrorMessage } from "../../../foundation/errors/error-format";
import { updateMcpConnection } from "../../../foundation/extensions/connection-operations";
import { accountQuery } from "../../../foundation/extensions/extension-query";
import type { McpConnection } from "../../../shared/contracts/generated/McpConnection";
import { useExtensionChanges } from "../use-extension-changes";

const props = defineProps<{ connection: McpConnection }>();
const query = useQuery(accountQuery);
const changes = useExtensionChanges();
const accounts = computed(() =>
  (query.data.value ?? []).filter((account) => account.connectorId === props.connection.connectorId),
);
function bind(event: Event) {
  const id = (event.target as HTMLSelectElement).value;
  void changes
    .mutateAsync(() =>
      updateMcpConnection({ ...props.connection, accountId: id || null, enabled: false, tools: [] }),
    )
    .catch(() => {});
}
</script>

<template>
  <section class="space-y-1">
    <label class="block text-caption"
      >{{ $t("extensions.selectedAccount")
      }}<select
        :value="connection.accountId ?? ''"
        class="ml-2 rounded-sm bg-surface-control p-2"
        :disabled="query.isPending.value || changes.isPending.value"
        data-testid="select-connector-account"
        @change="bind"
      >
        <option value="">{{ $t("extensions.noAccount") }}</option>
        <option
          v-for="account in accounts"
          :key="account.id"
          :value="account.id"
          :disabled="!account.enabled || account.cleanupPending"
        >
          {{ account.displayName }}
        </option>
      </select></label
    >
    <p class="text-caption text-fg-secondary">{{ $t("extensions.accountBindingChanges") }}</p>
    <p v-if="query.error.value || changes.error.value" role="alert" class="text-caption text-fg-danger">
      {{ getErrorMessage(query.error.value || changes.error.value) }}
    </p>
  </section>
</template>
