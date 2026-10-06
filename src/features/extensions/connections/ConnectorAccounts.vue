<script setup lang="ts">
import { useQuery } from "@tanstack/vue-query";
import { computed, shallowRef } from "vue";

import { getErrorMessage } from "../../../foundation/errors/error-format";
import {
  createConnectorAccount,
  updateMcpConnection,
} from "../../../foundation/extensions/connection-operations";
import { accountQuery } from "../../../foundation/extensions/extension-query";
import type { McpConnection } from "../../../shared/contracts/generated/McpConnection";
import UiButton from "../../../shared/ui/UiButton.vue";
import UiInput from "../../../shared/ui/UiInput.vue";
import { useExtensionChanges } from "../use-extension-changes";
import ConnectorAuthorization from "./ConnectorAuthorization.vue";
const props = defineProps<{ connection: McpConnection }>();
const query = useQuery(accountQuery);
const changes = useExtensionChanges();
const displayName = shallowRef("");
const accounts = computed(() =>
  (query.data.value ?? []).filter((account) => account.connectorId === props.connection.connectorId),
);
function create() {
  void changes
    .mutateAsync(async () => {
      await createConnectorAccount(props.connection.id, displayName.value);
      displayName.value = "";
    })
    .catch(() => {});
}
function bind(event: Event) {
  const id = (event.target as HTMLSelectElement).value;
  void changes
    .mutateAsync(() => updateMcpConnection({ ...props.connection, accountId: id || null }))
    .catch(() => {});
}
</script>

<template>
  <section class="space-y-3">
    <form class="flex items-end gap-2" @submit.prevent="create">
      <label class="min-w-0 flex-1 text-caption"
        >{{ $t("extensions.accountDisplayName")
        }}<UiInput v-model="displayName" data-testid="connector-account-name" /></label
      ><UiButton type="submit" :disabled="!displayName || changes.isPending.value">{{
        $t("extensions.addAccount")
      }}</UiButton>
    </form>
    <label class="block text-caption"
      >{{ $t("extensions.selectedAccount")
      }}<select
        :value="connection.accountId ?? ''"
        class="ml-2 rounded-sm bg-surface-control p-2"
        :disabled="changes.isPending.value"
        data-testid="select-connector-account"
        @change="bind"
      >
        <option value="">{{ $t("extensions.noAccount") }}</option>
        <option v-for="account in accounts" :key="account.id" :value="account.id">
          {{ account.displayName }}
        </option>
      </select></label
    >
    <p v-if="query.error.value || changes.error.value" role="alert" class="text-caption text-fg-danger">
      {{ getErrorMessage(query.error.value || changes.error.value) }}
    </p>
    <ConnectorAuthorization v-for="account in accounts" :key="account.id" :account="account" />
  </section>
</template>
