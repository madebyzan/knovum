<script setup lang="ts">
import { useQuery, useQueryClient } from "@tanstack/vue-query";
import { computed, onBeforeUnmount, onScopeDispose, reactive, shallowRef } from "vue";
import { useI18n } from "vue-i18n";

import { getErrorMessage } from "../../../../foundation/errors/error-format";
import {
  cancelConnectorAuthorization,
  connectConnector,
  disconnectConnector,
} from "../../../../foundation/extensions/connector-operations";
import { accountQuery, EXTENSION_QUERY_KEY } from "../../../../foundation/extensions/extension-query";
import type { ConnectorDefinition } from "../../../../shared/contracts/generated/ConnectorDefinition";
import { useSettingsLeaveContext } from "../../../../shared/contracts/settings-leave";
import { confirmAction } from "../../../../shared/ui/confirm-action";
import UiButton from "../../../../shared/ui/UiButton.vue";
import UiInput from "../../../../shared/ui/UiInput.vue";

/**
 * Connecting a service. A service is connected once, so this is a single action rather than a
 * list: there is nothing to name, nothing to pick and nothing to bind afterwards.
 */
const props = defineProps<{ connector: ConnectorDefinition }>();
const client = useQueryClient();
const { t } = useI18n();
const query = useQuery(accountQuery);
const fields = reactive<Record<string, string>>({});
const busy = shallowRef(false);
const error = shallowRef("");
const account = computed(() =>
  (query.data.value ?? []).find(({ connectorId }) => connectorId === props.connector.id),
);
const names = computed(() =>
  props.connector.authorization.type === "bearer"
    ? ["token"]
    : props.connector.authorization.type === "secret_fields"
      ? props.connector.authorization.names
      : [],
);
const canConnect = computed(() => !busy.value && names.value.every((name) => Boolean(fields[name]?.trim())));
const connected = computed(() => Boolean(account.value?.enabled));
const unregister = useSettingsLeaveContext().register({
  async prepare() {
    if (busy.value) return false;
    if (!Object.values(fields).some(Boolean)) return true;
    return confirmAction({
      title: t("extensions.accounts.discard.title"),
      description: t("extensions.accounts.discard.description"),
      confirmLabel: t("common.discardChanges"),
      danger: true,
    });
  },
  finalize: clear,
});
onScopeDispose(unregister);
function clear() {
  for (const name of Object.keys(fields)) delete fields[name];
}
onBeforeUnmount(clear);

async function run(operation: () => Promise<unknown>) {
  busy.value = true;
  error.value = "";
  try {
    await operation();
  } catch (cause) {
    error.value = getErrorMessage(cause);
  } finally {
    await client.invalidateQueries({ queryKey: EXTENSION_QUERY_KEY });
    busy.value = false;
  }
}

async function connect() {
  if (!canConnect.value) return;
  const values = Object.fromEntries(names.value.map((name) => [name, fields[name] ?? ""]));
  clear();
  await run(() => connectConnector(props.connector.id, values));
  for (const name of Object.keys(values)) delete values[name];
}

async function disconnect() {
  if (busy.value) return;
  clear();
  await run(() => disconnectConnector(props.connector.id));
}

async function cancel() {
  try {
    await cancelConnectorAuthorization(props.connector.id);
  } catch (cause) {
    error.value = getErrorMessage(cause);
  }
}
</script>

<template>
  <form class="space-y-3" data-testid="connector-authorization" @submit.prevent="connect">
    <p v-if="account" class="text-caption break-all text-fg-secondary">{{ account.resource }}</p>
    <p class="text-caption" :class="connected ? 'text-fg-secondary' : 'text-fg-warning'">
      {{
        $t(
          account?.cleanupPending
            ? "extensions.accounts.cleanupPending"
            : connected
              ? "extensions.accounts.authorized"
              : "extensions.accounts.required",
        )
      }}
    </p>
    <p v-if="account?.issuer" class="text-caption text-fg-secondary">
      {{ account.issuer }} · {{ account.scopes.join(" ") }}
    </p>
    <label v-for="name in names" :key="name" class="block space-y-1.5 text-caption text-fg-secondary"
      >{{ name
      }}<UiInput
        :model-value="fields[name] ?? ''"
        @update:model-value="fields[name] = $event"
        type="password"
        autocomplete="off"
        :disabled="busy"
    /></label>
    <p v-if="busy" role="status" class="text-caption text-fg-secondary">
      {{ $t("extensions.accounts.pending") }}
    </p>
    <p v-if="error" role="alert" class="text-caption text-fg-danger">{{ error }}</p>
    <div class="flex gap-2">
      <UiButton
        type="submit"
        size="sm"
        :variant="connected ? 'ghost' : 'primary'"
        :disabled="!canConnect"
        data-testid="connect-connector"
        >{{ $t(connected ? "extensions.accounts.reconnect" : "extensions.accounts.connect") }}</UiButton
      >
      <UiButton
        v-if="busy && connector.authorization.type === 'oauth'"
        size="sm"
        variant="ghost"
        @click="cancel"
        >{{ $t("common.cancel") }}</UiButton
      >
      <UiButton
        v-if="account"
        size="sm"
        variant="dangerSubtle"
        :disabled="busy"
        data-testid="disconnect-connector"
        @click="disconnect"
        >{{ $t("extensions.accounts.disconnect") }}</UiButton
      >
    </div>
  </form>
</template>
