<script setup lang="ts">
import { computed, shallowRef } from "vue";
import { z } from "zod";

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
const publicFields = defineModel<string>("publicFields", { required: true });
const fieldError = shallowRef("");
const fieldsLabel = computed(() =>
  connector.value.endpoint.type === "streamable_http"
    ? "extensions.connections.configuration.headers"
    : "extensions.connections.configuration.environment",
);
const publicFieldsSchema = z.record(z.string(), z.string());
function save() {
  let fields: Record<string, string>;
  try {
    fields = publicFieldsSchema.parse(JSON.parse(publicFields.value));
  } catch {
    fieldError.value = "extensions.connections.configuration.invalid";
    return;
  }
  const endpoint = connector.value.endpoint;
  connector.value = {
    ...connector.value,
    endpoint:
      endpoint.type === "streamable_http"
        ? { ...endpoint, headers: fields }
        : { ...endpoint, environment: fields },
  };
  fieldError.value = "";
  emit("save");
}
</script>

<template>
  <form class="space-y-4" data-testid="connector-editor" @submit.prevent="save">
    <label class="block space-y-1.5 text-caption text-fg-secondary"
      >{{ $t("extensions.connectors.name")
      }}<UiInput :disabled="busy" v-model="connector.name" data-testid="connector-name"
    /></label>
    <label class="block space-y-1.5 text-caption text-fg-secondary"
      >{{ $t("extensions.connectors.descriptionField") }}
      <UiTextarea :disabled="busy" v-model="connector.description" data-testid="connector-description" />
    </label>
    <McpEndpointFields :disabled="busy" v-model="connector.endpoint" />
    <details class="space-y-2 text-caption text-fg-secondary" :open="Boolean(fieldError)">
      <summary
        class="w-fit cursor-default rounded-sm py-1 outline-none focus-visible:outline-2 focus-visible:outline-stroke-focus"
      >
        {{ $t("extensions.connections.configuration.advanced") }}
      </summary>
      <label class="block space-y-1.5 text-caption text-fg-secondary"
        >{{ $t(fieldsLabel)
        }}<UiTextarea :disabled="busy" v-model="publicFields" class="font-mono text-caption"
      /></label>
      <p class="text-caption text-fg-secondary">
        {{ $t("extensions.connections.configuration.secretsHint") }}
      </p>
    </details>
    <ConnectorRequirements
      :disabled="busy"
      v-model="connector.authorization"
      :http="connector.endpoint.type === 'streamable_http'"
    />
    <p v-if="fieldError" role="alert" class="text-caption text-fg-danger">{{ $t(fieldError) }}</p>
    <div class="flex gap-2">
      <UiButton
        variant="primary"
        type="submit"
        :disabled="busy || !connector.name.trim() || !connector.description.trim()"
        data-testid="save-connector"
        >{{ $t("common.save") }}</UiButton
      ><UiButton :disabled="busy" variant="ghost" @click="emit('close')">{{ $t("common.close") }}</UiButton>
    </div>
  </form>
</template>
