<script setup lang="ts">
import { useQuery } from "@tanstack/vue-query";
import { computed, shallowRef } from "vue";

import { getErrorMessage } from "../../../foundation/errors/error-format";
import { createConnectorAccount } from "../../../foundation/extensions/connector-operations";
import { accountQuery } from "../../../foundation/extensions/extension-query";
import type { ConnectorDefinition } from "../../../shared/contracts/generated/ConnectorDefinition";
import UiButton from "../../../shared/ui/UiButton.vue";
import UiInput from "../../../shared/ui/UiInput.vue";
import { useExtensionChanges } from "../use-extension-changes";
import ConnectorAuthorization from "./ConnectorAuthorization.vue";
const props = defineProps<{ connector: ConnectorDefinition }>();
const query = useQuery(accountQuery);
const changes = useExtensionChanges();
const displayName = shallowRef("");
const accounts = computed(() =>
  (query.data.value ?? []).filter((account) => account.connectorId === props.connector.id),
);
function create() {
  void changes
    .mutateAsync(async () => {
      await createConnectorAccount(props.connector.id, displayName.value);
      displayName.value = "";
    })
    .catch(() => {});
}
</script>

<template>
  <section class="space-y-3">
    <form class="flex items-end gap-2" @submit.prevent="create">
      <label class="min-w-0 flex-1 space-y-1.5 text-caption text-fg-secondary"
        >{{ $t("extensions.accountDisplayName")
        }}<UiInput v-model="displayName" data-testid="connector-account-name" /></label
      ><UiButton type="submit" :disabled="!displayName.trim() || changes.isPending.value">{{
        $t("extensions.addAccount")
      }}</UiButton>
    </form>
    <p v-if="query.error.value || changes.error.value" role="alert" class="text-caption text-fg-danger">
      {{ getErrorMessage(query.error.value || changes.error.value) }}
    </p>
    <ConnectorAuthorization v-for="account in accounts" :key="account.id" :account="account" />
  </section>
</template>
