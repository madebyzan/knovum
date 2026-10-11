<script setup lang="ts">
import { shallowRef } from "vue";

import type { ConnectorDefinition } from "../../../../shared/contracts/generated/ConnectorDefinition";
import type { McpEndpoint } from "../../../../shared/contracts/generated/McpEndpoint";
import UiInput from "../../../../shared/ui/UiInput.vue";
import McpEndpointFields from "../../connections/components/McpEndpointFields.vue";
import McpPublicFields from "../../connections/components/McpPublicFields.vue";
import { validateEndpointFields, type EndpointField } from "../../connections/endpoint-fields";
import ConnectorRequirements from "./ConnectorRequirements.vue";
const connector = defineModel<ConnectorDefinition & { endpoint: McpEndpoint }>({ required: true });
const endpointFields = defineModel<EndpointField[]>("endpointFields", { required: true });
defineProps<{ busy: boolean; formId: string }>();
const emit = defineEmits<{ save: [] }>();
const fieldError = shallowRef("");
function save() {
  const validatedFields = validateEndpointFields(endpointFields.value, connector.value.endpoint.type);
  if ("error" in validatedFields) {
    fieldError.value = validatedFields.error;
    return;
  }
  const endpoint = connector.value.endpoint;
  connector.value = {
    ...connector.value,
    endpoint:
      endpoint.type === "streamable_http"
        ? { ...endpoint, headers: validatedFields.fields }
        : { ...endpoint, environment: validatedFields.fields },
  };
  fieldError.value = "";
  emit("save");
}
function changeTransport() {
  connector.value.authorization = { type: "none" };
  fieldError.value = "";
}
</script>
<template>
  <form :id="formId" class="space-y-4" data-testid="connector-editor" @submit.prevent="save">
    <section class="space-y-3">
      <label class="block space-y-1.5 text-caption text-fg-secondary">
        {{ $t("extensions.connectors.name") }}
        <UiInput v-model="connector.name" :disabled="busy" data-testid="connector-name" required />
      </label>
      <label class="block space-y-1.5 text-caption text-fg-secondary">
        {{ $t("extensions.connectors.descriptionField") }}
        <UiInput v-model="connector.description" :disabled="busy" data-testid="connector-description" />
      </label>
    </section>
    <section class="space-y-4 border-t border-stroke-divider pt-4">
      <McpEndpointFields v-model="connector.endpoint" :disabled="busy" @change-transport="changeTransport" />
      <details class="group space-y-3" :open="endpointFields.length > 0">
        <summary class="cursor-default text-label text-fg-secondary">
          {{ $t("extensions.endpoint.configuration.advanced") }}
        </summary>
        <McpPublicFields v-model="endpointFields" :transport="connector.endpoint.type" :disabled="busy" />
      </details>
      <p v-if="fieldError" role="alert" class="text-caption text-fg-danger">{{ $t(fieldError) }}</p>
    </section>
    <section class="space-y-2 border-t border-stroke-divider pt-4">
      <ConnectorRequirements
        v-model="connector.authorization"
        :disabled="busy"
        :http="connector.endpoint.type === 'streamable_http'"
      />
      <p class="text-caption text-fg-secondary">
        {{ $t("extensions.endpoint.configuration.authorizationHint") }}
      </p>
    </section>
  </form>
</template>
