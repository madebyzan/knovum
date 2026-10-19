<script setup lang="ts">
import { Check, Code2, Compass, Copy } from "@lucide/vue";
import { useMutation } from "@tanstack/vue-query";
import { computed, ref, shallowRef } from "vue";

import { getErrorMessage } from "../../../../foundation/errors/error-format";
import { promptArguments, resourceParameters } from "../../../../foundation/extensions/catalog-input";
import { getMcpPrompt, readMcpResource } from "../../../../foundation/extensions/connection-operations";
import type { McpConnection } from "../../../../shared/contracts/generated/McpConnection";
import type { McpConnectionCatalog } from "../../../../shared/contracts/generated/McpConnectionCatalog";
import type { McpResourceSelection } from "../../../../shared/contracts/generated/McpResourceSelection";
import UiButton from "../../../../shared/ui/UiButton.vue";
import UiSelect from "../../../../shared/ui/UiSelect.vue";
import UiSpinner from "../../../../shared/ui/UiSpinner.vue";
import UiTextarea from "../../../../shared/ui/UiTextarea.vue";

const props = defineProps<{ connection: McpConnection; catalog: McpConnectionCatalog }>();
const kind = ref<"resource" | "template" | "prompt">("resource");
const name = ref("");
const parameters = ref("{}");
const content = shallowRef<unknown>();
const copied = shallowRef(false);

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

async function copyOutput() {
  if (!content.value) return;
  await navigator.clipboard.writeText(displayContent(content.value));
  copied.value = true;
  setTimeout(() => {
    copied.value = false;
  }, 1500);
}
</script>

<template>
  <section
    class="space-y-4 rounded-panel border border-stroke-default bg-surface-raised p-4"
    data-testid="mcp-catalog-browser"
  >
    <div class="flex items-center gap-2">
      <span
        class="flex size-6 shrink-0 items-center justify-center rounded bg-surface-control text-fg-secondary"
      >
        <Compass class="size-3.5" />
      </span>
      <h3 class="text-body font-semibold text-fg-primary">{{ $t("extensions.catalog.browse") }}</h3>
    </div>

    <form class="space-y-3" @submit.prevent="reading.mutate()">
      <div class="grid grid-cols-1 gap-3 sm:grid-cols-2">
        <div class="space-y-1">
          <span class="text-caption text-fg-secondary">{{ $t("extensions.catalog.kind") }}</span>
          <UiSelect
            v-model="kind"
            size="sm"
            :disabled="reading.isPending.value"
            :label="$t('extensions.catalog.kind')"
            :options="[
              { value: 'resource', label: $t('extensions.catalog.resources') },
              { value: 'template', label: $t('extensions.catalog.templates') },
              { value: 'prompt', label: $t('extensions.catalog.prompts') },
            ]"
            @update:model-value="change"
          />
        </div>
        <div class="space-y-1">
          <span class="text-caption text-fg-secondary">{{ $t("extensions.catalog.item") }}</span>
          <UiSelect
            v-model="name"
            size="sm"
            :disabled="reading.isPending.value || !items.length"
            :label="$t('extensions.catalog.item')"
            :options="items"
          />
        </div>
      </div>

      <label v-if="kind !== 'resource'" class="block space-y-1 text-caption text-fg-secondary">
        <span>{{ $t("extensions.catalog.parameters") }}</span>
        <UiTextarea
          v-model="parameters"
          :disabled="reading.isPending.value"
          class="bg-surface-canvas font-mono text-caption"
          :rows="3"
        />
      </label>

      <UiButton size="sm" type="submit" :disabled="reading.isPending.value || !name || !connection.enabled">
        <UiSpinner v-if="reading.isPending.value" class="size-3.5" />
        <span>{{ $t("extensions.catalog.read") }}</span>
      </UiButton>
    </form>

    <p v-if="reading.error.value" role="alert" class="text-caption text-fg-danger">
      {{ getErrorMessage(reading.error.value) }}
    </p>

    <div v-if="content" class="overflow-hidden rounded-md border border-stroke-default bg-surface-canvas">
      <div
        class="flex items-center justify-between border-b border-stroke-divider bg-surface-control/40 px-3 py-1.5"
      >
        <div class="flex items-center gap-1.5 text-caption-sm text-fg-secondary">
          <Code2 class="size-3.5" />
          <span>JSON</span>
        </div>
        <UiButton size="sm" variant="ghost" class="h-6 px-1.5 text-caption-sm" @click="copyOutput">
          <component :is="copied ? Check : Copy" class="size-3" />
          <span>{{ copied ? $t("common.copied") : $t("common.copy") }}</span>
        </UiButton>
      </div>
      <pre
        class="max-h-80 overflow-auto p-3 font-mono text-caption leading-relaxed break-all whitespace-pre-wrap text-fg-primary select-text"
        >{{ displayContent(content) }}</pre>
    </div>
  </section>
</template>
