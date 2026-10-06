<script setup lang="ts">
import { computed } from "vue";

import type { ConnectorAuthorization } from "../../../shared/contracts/generated/ConnectorAuthorization";
import UiInput from "../../../shared/ui/UiInput.vue";
defineProps<{ http: boolean }>();
const authorization = defineModel<ConnectorAuthorization>({ required: true });
function select(event: Event) {
  const type = (event.target as HTMLSelectElement).value;
  if (type === "oauth") authorization.value = { type: "oauth", clientId: null, scopes: [] };
  else if (type === "secret_fields") authorization.value = { type: "secret_fields", names: [] };
  else authorization.value = type === "bearer" ? { type: "bearer" } : { type: "none" };
}
const publicClient = computed({
  get: () => (authorization.value.type === "oauth" ? (authorization.value.clientId ?? "") : ""),
  set: (value: string) => {
    if (authorization.value.type === "oauth")
      authorization.value = { ...authorization.value, clientId: value || null };
  },
});
const scopes = computed({
  get: () => (authorization.value.type === "oauth" ? authorization.value.scopes.join(" ") : ""),
  set: (value: string) => {
    if (authorization.value.type === "oauth")
      authorization.value = { ...authorization.value, scopes: value.split(/\s+/).filter(Boolean) };
  },
});
const names = computed({
  get: () => (authorization.value.type === "secret_fields" ? authorization.value.names.join(" ") : ""),
  set: (value: string) => {
    if (authorization.value.type === "secret_fields")
      authorization.value = { ...authorization.value, names: value.split(/\s+/).filter(Boolean) };
  },
});
</script>

<template>
  <div class="space-y-2">
    <label class="flex items-center gap-2 text-caption"
      >{{ $t("extensions.authorization")
      }}<select
        :value="authorization.type === 'oauth' ? 'oauth' : authorization.type"
        class="rounded-sm bg-surface-control p-1"
        @change="select"
      >
        <option value="none">{{ $t("extensions.noAuthorization") }}</option>
        <option value="bearer" :disabled="!http">API Token</option>
        <option value="secret_fields">{{ $t("extensions.secretFields") }}</option>
        <option value="oauth" :disabled="!http">OAuth + PKCE</option>
      </select></label
    >
    <template v-if="authorization.type === 'oauth'">
      <label class="block text-caption"
        >{{ $t("extensions.publicClientId") }}<UiInput v-model="publicClient"
      /></label>
      <label class="block text-caption">Scopes<UiInput v-model="scopes" /></label>
    </template>
    <label v-if="authorization.type === 'secret_fields'" class="block text-caption"
      >{{ $t("extensions.secretFieldNames") }}<UiInput v-model="names"
    /></label>
  </div>
</template>
