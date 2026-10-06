<script setup lang="ts">
import { useMutation } from "@tanstack/vue-query";
import { ref, shallowRef } from "vue";

import { getErrorMessage } from "../../../foundation/errors/error-format";
import { promptArguments, resourceParameters } from "../../../foundation/extensions/catalog-input";
import { getMcpPrompt, readMcpResource } from "../../../foundation/extensions/connection-operations";
import type { McpConnection } from "../../../shared/contracts/generated/McpConnection";
import type { McpConnectionCatalog } from "../../../shared/contracts/generated/McpConnectionCatalog";
import type { McpResourceSelection } from "../../../shared/contracts/generated/McpResourceSelection";
import UiButton from "../../../shared/ui/UiButton.vue";
const props = defineProps<{ connection: McpConnection; catalog: McpConnectionCatalog }>();
const kind = ref<"resource" | "template" | "prompt">("resource");
const name = ref("");
const parameters = ref("{}");
const content = shallowRef<unknown>();
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
  <section class="space-y-2 rounded-sm border border-stroke-divider p-3" data-testid="mcp-catalog-browser">
    <h3 class="text-body font-medium">{{ $t("extensions.browseResourcesAndPrompts") }}</h3>
    <form class="space-y-2" @submit.prevent="reading.mutate()">
      <label class="block text-caption"
        >{{ $t("extensions.catalogKind")
        }}<select
          v-model="kind"
          :disabled="reading.isPending.value"
          class="ml-2 bg-surface-control"
          @change="change"
        >
          <option value="resource">{{ $t("extensions.resources") }}</option>
          <option value="template">{{ $t("extensions.resourceTemplates") }}</option>
          <option value="prompt">{{ $t("extensions.prompts") }}</option>
        </select></label
      >
      <label class="block text-caption"
        >{{ $t("extensions.catalogItem")
        }}<select
          v-model="name"
          :disabled="reading.isPending.value"
          class="block w-full bg-surface-control"
          required
        >
          <option value="" />
          <template v-if="kind === 'resource'"
            ><option v-for="resource in catalog.resources" :key="resource.uri" :value="resource.uri">
              {{ resource.name }} · {{ resource.uri }}
            </option></template
          >
          <template v-if="kind === 'template'"
            ><option
              v-for="template in catalog.resourceTemplates"
              :key="template.uriTemplate"
              :value="template.uriTemplate"
            >
              {{ template.name }} · {{ template.uriTemplate }}
            </option></template
          >
          <template v-if="kind === 'prompt'"
            ><option v-for="prompt in catalog.prompts" :key="prompt.name" :value="prompt.name">
              {{ prompt.name }} · {{ prompt.description }}
            </option></template
          >
        </select></label
      >
      <label v-if="kind !== 'resource'" class="block text-caption"
        >{{ $t("extensions.parameters")
        }}<textarea
          v-model="parameters"
          :disabled="reading.isPending.value"
          class="block w-full rounded-sm bg-surface-control p-2 font-mono"
          rows="3"
        />
      </label>
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
