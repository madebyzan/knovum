<script setup lang="ts">
import { computed, shallowRef, watch } from "vue";

import type { ConnectorAuthorization } from "../../../shared/contracts/generated/ConnectorAuthorization";
import type { McpConnection } from "../../../shared/contracts/generated/McpConnection";
import UiButton from "../../../shared/ui/UiButton.vue";
import UiInput from "../../../shared/ui/UiInput.vue";
import UiTextarea from "../../../shared/ui/UiTextarea.vue";
import ConnectorRequirements from "./ConnectorRequirements.vue";
import McpEndpointFields from "./McpEndpointFields.vue";
const connection = defineModel<McpConnection>({ required: true });
const authorization = defineModel<ConnectorAuthorization>("authorization", { required: true });
defineProps<{ busy: boolean }>();
const emit = defineEmits<{ save: []; close: [] }>();
const publicFields = shallowRef("{}");
const fieldError = shallowRef("");
watch(
  () => connection.value.endpoint,
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
  connection.value.endpoint.type === "streamable_http"
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
    const endpoint = connection.value.endpoint;
    connection.value = {
      ...connection.value,
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
    data-testid="mcp-connection-editor"
    @submit.prevent="save"
  >
    <label class="block text-caption"
      >{{ $t("extensions.connectionName")
      }}<UiInput v-model="connection.name" data-testid="mcp-connection-name"
    /></label>
    <McpEndpointFields v-model="connection.endpoint" />
    <label class="block text-caption"
      >{{ $t(fieldsLabel) }}<UiTextarea v-model="publicFields" class="font-mono text-caption"
    /></label>
    <p class="text-caption text-fg-secondary">{{ $t("extensions.noSecretsInConfiguration") }}</p>
    <ConnectorRequirements v-model="authorization" />
    <p v-if="fieldError" role="alert" class="text-caption text-fg-danger">{{ $t(fieldError) }}</p>
    <div class="flex gap-2">
      <UiButton type="submit" :disabled="busy || !connection.name" data-testid="save-mcp-connection">{{
        $t("extensions.save")
      }}</UiButton
      ><UiButton variant="ghost" @click="emit('close')">{{ $t("extensions.close") }}</UiButton>
    </div>
  </form>
</template>
