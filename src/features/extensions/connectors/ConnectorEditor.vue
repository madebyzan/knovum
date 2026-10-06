<script setup lang="ts">
import { computed, shallowRef, watch } from "vue";

import type { ConnectorDefinition } from "../../../shared/contracts/generated/ConnectorDefinition";
import type { McpEndpoint } from "../../../shared/contracts/generated/McpEndpoint";
import UiButton from "../../../shared/ui/UiButton.vue";
import UiInput from "../../../shared/ui/UiInput.vue";
import UiTextarea from "../../../shared/ui/UiTextarea.vue";
import McpEndpointFields from "../connections/McpEndpointFields.vue";
import ConnectorRequirements from "./ConnectorRequirements.vue";
const connector = defineModel<ConnectorDefinition & { endpoint: McpEndpoint }>({ required: true });
defineProps<{ busy: boolean }>();
const emit = defineEmits<{ save: []; close: [] }>();
const publicFields = shallowRef("{}");
const fieldError = shallowRef("");
watch(
  () => connector.value.endpoint,
  (endpoint) => {
    publicFields.value = JSON.stringify(
      endpoint.type === "streamable_http" ? endpoint.headers : endpoint.environment,
      null,
      2,
    );
  },
  { immediate: true },
);
const fieldsLabel = computed(() =>
  connector.value.endpoint.type === "streamable_http"
    ? "extensions.publicHeaders"
    : "extensions.publicEnvironment",
);
function save() {
  try {
    const fields: unknown = JSON.parse(publicFields.value);
    if (
      !fields ||
      typeof fields !== "object" ||
      Array.isArray(fields) ||
      Object.values(fields).some((value) => typeof value !== "string")
    )
      throw new Error("String values are required");
    const endpoint = connector.value.endpoint;
    connector.value = {
      ...connector.value,
      endpoint:
        endpoint.type === "streamable_http"
          ? { ...endpoint, headers: fields as Record<string, string> }
          : { ...endpoint, environment: fields as Record<string, string> },
    };
    fieldError.value = "";
    emit("save");
  } catch {
    fieldError.value = "extensions.publicFieldsInvalid";
  }
}
</script>

<template>
  <form
    class="space-y-3 rounded-sm border border-stroke-divider p-3"
    data-testid="connector-editor"
    @submit.prevent="save"
  >
    <label class="block text-caption"
      >{{ $t("extensions.connectorName") }}<UiInput v-model="connector.name" data-testid="connector-name"
    /></label>
    <label class="block text-caption"
      >{{ $t("extensions.connectorDescription") }}
      <UiTextarea v-model="connector.description" data-testid="connector-description" />
    </label>
    <McpEndpointFields v-model="connector.endpoint" />
    <label class="block text-caption"
      >{{ $t(fieldsLabel) }}<UiTextarea v-model="publicFields" class="font-mono text-caption"
    /></label>
    <p class="text-caption text-fg-secondary">{{ $t("extensions.noSecretsInConfiguration") }}</p>
    <ConnectorRequirements
      v-model="connector.authorization"
      :http="connector.endpoint.type === 'streamable_http'"
    />
    <p v-if="fieldError" role="alert" class="text-caption text-fg-danger">{{ $t(fieldError) }}</p>
    <div class="flex gap-2">
      <UiButton
        type="submit"
        :disabled="busy || !connector.name.trim() || !connector.description.trim()"
        data-testid="save-connector"
        >{{ $t("extensions.save") }}</UiButton
      ><UiButton variant="ghost" @click="emit('close')">{{ $t("extensions.close") }}</UiButton>
    </div>
  </form>
</template>
