<script setup lang="ts">
import { useQuery } from "@tanstack/vue-query";
import { computed, onScopeDispose, shallowRef } from "vue";
import { useI18n } from "vue-i18n";

import { getErrorMessage } from "../../../foundation/errors/error-format";
import { createConnectorAccount } from "../../../foundation/extensions/connector-operations";
import { accountQuery } from "../../../foundation/extensions/extension-query";
import type { ConnectorDefinition } from "../../../shared/contracts/generated/ConnectorDefinition";
import { useSettingsLeaveContext } from "../../../shared/contracts/settings-leave";
import { confirmAction } from "../../../shared/ui/confirm-action";
import UiButton from "../../../shared/ui/UiButton.vue";
import UiInput from "../../../shared/ui/UiInput.vue";
import { useExtensionChanges } from "../use-extension-changes";
import ConnectorAuthorization from "./ConnectorAuthorization.vue";
const props = defineProps<{ connector: ConnectorDefinition }>();
const query = useQuery(accountQuery);
const { t } = useI18n();
const changes = useExtensionChanges();
const displayName = shallowRef("");
const accounts = computed(() =>
  (query.data.value ?? []).filter((account) => account.connectorId === props.connector.id),
);
let admittedCreation: Promise<boolean> | undefined;
const unregister = useSettingsLeaveContext().register({
  async prepare() {
    if (admittedCreation && !(await admittedCreation)) return false;
    if (!displayName.value.trim()) return true;
    return confirmAction({
      title: t("extensions.accounts.discard.title"),
      description: t("extensions.accounts.discard.description"),
      confirmLabel: t("common.discardChanges"),
      danger: true,
    });
  },
  finalize() {
    displayName.value = "";
  },
});
onScopeDispose(unregister);
function create() {
  if (changes.isPending.value) return;
  const admitted: Promise<boolean> = changes
    .mutateAsync(async () => {
      await createConnectorAccount(props.connector.id, displayName.value);
      displayName.value = "";
    })
    .then(
      () => true,
      () => false,
    )
    .finally(() => {
      if (admittedCreation === admitted) admittedCreation = undefined;
    });
  admittedCreation = admitted;
}
</script>

<template>
  <section class="space-y-3">
    <form class="flex items-end gap-2" @submit.prevent="create">
      <label class="min-w-0 flex-1 space-y-1.5 text-caption text-fg-secondary"
        >{{ $t("extensions.accounts.name")
        }}<UiInput
          v-model="displayName"
          :disabled="changes.isPending.value"
          data-testid="connector-account-name" /></label
      ><UiButton type="submit" :disabled="!displayName.trim() || changes.isPending.value">{{
        $t("extensions.accounts.add")
      }}</UiButton>
    </form>
    <p v-if="query.error.value || changes.error.value" role="alert" class="text-caption text-fg-danger">
      {{ getErrorMessage(query.error.value || changes.error.value) }}
    </p>
    <ConnectorAuthorization v-for="account in accounts" :key="account.id" :account="account" />
  </section>
</template>
