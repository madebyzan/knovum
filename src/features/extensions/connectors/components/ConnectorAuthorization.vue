<script setup lang="ts">
import { useQueryClient } from "@tanstack/vue-query";
import { computed, onBeforeUnmount, onScopeDispose, reactive, shallowRef } from "vue";
import { useI18n } from "vue-i18n";

import { getErrorMessage } from "../../../../foundation/errors/error-format";
import {
  authorizeConnectorAccount,
  cancelConnectorAuthorization,
  disconnectConnectorAccount,
} from "../../../../foundation/extensions/connector-operations";
import { extensionQueryKey } from "../../../../foundation/extensions/extension-query";
import type { ConnectorAccount } from "../../../../shared/contracts/generated/ConnectorAccount";
import { useSettingsLeaveContext } from "../../../../shared/contracts/settings-leave";
import { confirmAction } from "../../../../shared/ui/confirm-action";
import UiButton from "../../../../shared/ui/UiButton.vue";
import UiInput from "../../../../shared/ui/UiInput.vue";
const props = defineProps<{ account: ConnectorAccount }>();
const client = useQueryClient();
const { t } = useI18n();
const fields = reactive<Record<string, string>>({});
const busy = shallowRef(false);
const error = shallowRef("");
const names = computed(() =>
  props.account.authorization.type === "bearer"
    ? ["token"]
    : props.account.authorization.type === "secret_fields"
      ? props.account.authorization.names
      : [],
);
const canAuthorize = computed(
  () => !busy.value && names.value.every((name) => Boolean(fields[name]?.trim())),
);
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
async function authorize() {
  if (!canAuthorize.value) return;
  busy.value = true;
  error.value = "";
  const values = Object.fromEntries(names.value.map((name) => [name, fields[name] ?? ""]));
  clear();
  try {
    await authorizeConnectorAccount(props.account.id, values);
  } catch (cause) {
    error.value = getErrorMessage(cause);
  } finally {
    for (const name of Object.keys(values)) delete values[name];
    await client.invalidateQueries({ queryKey: extensionQueryKey });
    busy.value = false;
  }
}
async function disconnect() {
  if (busy.value) return;
  clear();
  busy.value = true;
  error.value = "";
  try {
    await disconnectConnectorAccount(props.account.id);
  } catch (cause) {
    error.value = getErrorMessage(cause);
  } finally {
    await client.invalidateQueries({ queryKey: extensionQueryKey });
    busy.value = false;
  }
}
async function cancel() {
  try {
    await cancelConnectorAuthorization(props.account.id);
  } catch (cause) {
    error.value = getErrorMessage(cause);
  }
}
</script>

<template>
  <form
    class="space-y-3 border-b border-stroke-divider py-4"
    data-testid="connector-authorization"
    @submit.prevent="authorize"
  >
    <h3 class="text-body font-medium">{{ account.displayName }}</h3>
    <p class="text-caption break-all text-fg-secondary">{{ account.resource }}</p>
    <p class="text-caption" :class="account.enabled ? 'text-fg-secondary' : 'text-fg-warning'">
      {{
        $t(
          account.cleanupPending
            ? "extensions.accounts.cleanupPending"
            : account.enabled
              ? "extensions.accounts.authorized"
              : "extensions.accounts.required",
        )
      }}
    </p>
    <p v-if="account.issuer" class="text-caption text-fg-secondary">
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
      <UiButton type="submit" size="sm" :disabled="!canAuthorize" data-testid="authorize-connector">{{
        $t(
          account.authorization.type === "oauth"
            ? "extensions.accounts.authorizeInBrowser"
            : "extensions.accounts.authorize",
        )
      }}</UiButton>
      <UiButton
        v-if="busy && account.authorization.type === 'oauth'"
        size="sm"
        variant="ghost"
        @click="cancel"
        >{{ $t("common.cancel") }}</UiButton
      >
      <UiButton
        v-if="account.enabled || account.cleanupPending"
        size="sm"
        variant="dangerSubtle"
        :disabled="busy"
        @click="disconnect"
        >{{ $t("extensions.accounts.disconnect") }}</UiButton
      >
    </div>
  </form>
</template>
