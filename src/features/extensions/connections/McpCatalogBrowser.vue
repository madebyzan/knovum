<script setup lang="ts">
import { useMutation } from "@tanstack/vue-query";
import { computed, ref, shallowRef } from "vue";

import { getErrorMessage } from "../../../foundation/errors/error-format";
import { promptArguments, resourceParameters } from "../../../foundation/extensions/catalog-input";
import { getMcpPrompt, readMcpResource } from "../../../foundation/extensions/connection-operations";
import type { McpConnection } from "../../../shared/contracts/generated/McpConnection";
import type { McpConnectionCatalog } from "../../../shared/contracts/generated/McpConnectionCatalog";
import type { McpResourceSelection } from "../../../shared/contracts/generated/McpResourceSelection";
import UiButton from "../../../shared/ui/UiButton.vue";
import UiSelect from "../../../shared/ui/UiSelect.vue";
import UiTextarea from "../../../shared/ui/UiTextarea.vue";
const props = defineProps<{ connection: McpConnection; catalog: McpConnectionCatalog }>();
const kind = ref<"resource" | "template" | "prompt">("resource");
const name = ref("");
const parameters = ref("{}");
const content = shallowRef<unknown>();
const items = computed(() => {
  if (kind.value === "resource")
    return props.catalog.resources.map((item) => ({ value: item.uri, label: `${item.name} · ${item.uri}` }));
  if (kind.value === "template")
    return props.catalog.resourceTemplates.map((item) => ({
      value: item.uriTemplate,
      label: `${item.name} · ${item.uriTemplate}`,
    }));
  return props.catalog.prompts.map((item) => ({
    value: item.name,
    label: item.name,
    description: item.description ?? undefined,
  }));
});
const reading = useMutation({
  mutationFn: async () => {
    if (kind.value === "prompt")
      return getMcpPrompt(
        props.connection,
        props.catalog.digest,
        name.value,
        promptArguments(parameters.value),
      );
    const selection: McpResourceSelection =
      kind.value === "template"
        ? {
            type: "template",
            selection: { uriTemplate: name.value, parameters: resourceParameters(parameters.value) },
          }
        : { type: "resource", uri: name.value };
    return readMcpResource(props.connection, props.catalog.digest, selection);
  },
  onSuccess: (value) => {
    content.value = value;
  },
});
function displayContent(value: unknown) {
  return JSON.stringify(
    value,
    (key, field: unknown) =>
      (key === "blob" || key === "data") && typeof field === "string"
        ? `[binary content: ${field.length} base64 characters]`
        : field,
    2,
  );
}
function change() {
  name.value = "";
  content.value = undefined;
  reading.reset();
}
</script>

<template>
  <section class="space-y-3" data-testid="mcp-catalog-browser">
    <h3 class="text-body font-medium">{{ $t("extensions.browseResourcesAndPrompts") }}</h3>
    <form class="space-y-2" @submit.prevent="reading.mutate()">
      <div class="space-y-1.5">
        <span class="text-caption text-fg-secondary">{{ $t("extensions.catalogKind") }}</span
        ><UiSelect
          v-model="kind"
          :disabled="reading.isPending.value"
          :label="$t('extensions.catalogKind')"
          :options="[
            { value: 'resource', label: $t('extensions.resources') },
            { value: 'template', label: $t('extensions.resourceTemplates') },
            { value: 'prompt', label: $t('extensions.prompts') },
          ]"
          @update:model-value="change"
        />
      </div>
      <div class="space-y-1.5">
        <span class="text-caption text-fg-secondary">{{ $t("extensions.catalogItem") }}</span
        ><UiSelect
          v-model="name"
          :disabled="reading.isPending.value || !items.length"
          :label="$t('extensions.catalogItem')"
          :options="items"
        />
      </div>
      <label v-if="kind !== 'resource'" class="block space-y-1.5 text-caption text-fg-secondary"
        >{{ $t("extensions.parameters")
        }}<UiTextarea v-model="parameters" :disabled="reading.isPending.value" class="font-mono" :rows="3"
      /></label>
      <UiButton size="sm" type="submit" :disabled="reading.isPending.value || !name || !connection.enabled">{{
        $t("extensions.read")
      }}</UiButton>
    </form>
    <p v-if="reading.error.value" role="alert" class="text-caption text-fg-danger">
      {{ getErrorMessage(reading.error.value) }}
    </p>
    <pre v-if="content" class="max-h-80 overflow-auto text-caption break-all whitespace-pre-wrap">{{
      displayContent(content)
    }}</pre>
  </section>
</template>
