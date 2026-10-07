<script setup lang="ts">
import { computed } from "vue";

import type { ConnectorAuthorization } from "../../../shared/contracts/generated/ConnectorAuthorization";
import UiInput from "../../../shared/ui/UiInput.vue";
import UiSelect from "../../../shared/ui/UiSelect.vue";
defineProps<{ http: boolean; disabled?: boolean }>();
const authorization = defineModel<ConnectorAuthorization>({ required: true });
function select(type: ConnectorAuthorization["type"]) {
  if (type === "oauth") authorization.value = { type: "oauth", clientId: null, scopes: [] };
  else if (type === "secret_fields") authorization.value = { type: "secret_fields", names: [] };
  else authorization.value = type === "bearer" ? { type: "bearer" } : { type: "none" };
}
const type = computed({ get: () => authorization.value.type, set: select });
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
  <div class="space-y-4">
    <div class="space-y-1.5">
      <span class="text-caption text-fg-secondary">{{ $t("extensions.authorization") }}</span
      ><UiSelect
        v-model="type"
        :disabled="disabled"
        :label="$t('extensions.authorization')"
        :options="[
          { value: 'none', label: $t('extensions.noAuthorization') },
          { value: 'bearer', label: 'API Token', disabled: !http },
          { value: 'secret_fields', label: $t('extensions.secretFields') },
          { value: 'oauth', label: 'OAuth + PKCE', disabled: !http },
        ]"
      />
    </div>
    <template v-if="authorization.type === 'oauth'">
      <label class="block space-y-1.5 text-caption text-fg-secondary"
        >{{ $t("extensions.publicClientId") }}<UiInput :disabled="disabled" v-model="publicClient"
      /></label>
      <label class="block space-y-1.5 text-caption text-fg-secondary"
        >Scopes<UiInput :disabled="disabled" v-model="scopes"
      /></label>
    </template>
    <label
      v-if="authorization.type === 'secret_fields'"
      class="block space-y-1.5 text-caption text-fg-secondary"
      >{{ $t("extensions.secretFieldNames") }}<UiInput :disabled="disabled" v-model="names"
    /></label>
  </div>
</template>
