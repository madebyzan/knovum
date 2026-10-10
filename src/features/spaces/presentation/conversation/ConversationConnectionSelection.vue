<script setup lang="ts">
import { useMutation } from "@tanstack/vue-query";
import { computed } from "vue";

import { getErrorMessage } from "../../../../foundation/errors/error-format";
import { checkMcpConnection } from "../../../../foundation/extensions/connection-operations";
import type { AgentConnectionSelection } from "../../../../shared/contracts/generated/AgentConnectionSelection";
import type { AgentExtensionSelection } from "../../../../shared/contracts/generated/AgentExtensionSelection";
import type { McpConnection } from "../../../../shared/contracts/generated/McpConnection";
import type { McpResourceTemplateSelection } from "../../../../shared/contracts/generated/McpResourceTemplateSelection";
import UiButton from "../../../../shared/ui/UiButton.vue";
import UiCheckbox from "../../../../shared/ui/UiCheckbox.vue";
import McpResourcePreview from "./McpResourcePreview.vue";
import McpResourceTemplateForm from "./McpResourceTemplateForm.vue";
const selection = defineModel<AgentExtensionSelection>({ required: true });
const props = defineProps<{ connection: McpConnection; disabled: boolean }>();
const choices = computed(() =>
  selection.value.connections.find((candidate) => candidate.connectionId === props.connection.id)!,
);
const availableTools = computed(() => props.connection.tools.filter((tool) => tool.policy !== "deny"));
const catalog = useMutation({
  mutationFn: () => checkMcpConnection(props.connection.id),
  networkMode: "online",
});
const resources = computed(() => catalog.data.value?.resources ?? []);
const prompts = computed(() => catalog.data.value?.prompts.map((prompt) => prompt.name) ?? []);
function toggle(kind: "toolNames" | "resourceUris" | "promptNames", value: string) {
  const values = choices.value[kind];
  const updated: AgentConnectionSelection = {
    ...choices.value,
    [kind]: values.includes(value) ? values.filter((item) => item !== value) : [...values, value],
  };
  selection.value = {
    ...selection.value,
    connections: selection.value.connections.map((connection) =>
      connection.connectionId === updated.connectionId ? updated : connection,
    ),
  };
}
function addTemplate(template: McpResourceTemplateSelection) {
  selection.value = {
    ...selection.value,
    connections: selection.value.connections.map((connection) =>
      connection.connectionId === props.connection.id
        ? Object.assign({}, connection, { resourceTemplates: [...connection.resourceTemplates, template] })
        : connection,
    ),
  };
}
function removeTemplate(index: number) {
  selection.value = {
    ...selection.value,
    connections: selection.value.connections.map((connection) =>
      connection.connectionId === props.connection.id
        ? Object.assign({}, connection, {
            resourceTemplates: connection.resourceTemplates.filter((_, position) => position !== index),
          })
        : connection,
    ),
  };
}
</script>

<template>
  <fieldset :disabled="disabled" class="space-y-2 border-t border-stroke-divider pt-2 text-caption">
    <h4 class="font-medium">{{ connection.name }}</h4>
    <label v-for="tool in availableTools" :key="tool.name" class="flex items-center gap-2"
      ><UiCheckbox
        :disabled="disabled"
        :label="tool.name"
        :model-value="choices.toolNames.includes(tool.name)"
        @update:model-value="toggle('toolNames', tool.name)"
      />{{ tool.name }}</label
    >
    <UiButton
      size="sm"
      variant="ghost"
      :disabled="disabled || catalog.isPending.value"
      @click="catalog.mutate()"
      >{{ $t("extensions.catalog.browse") }}</UiButton
    >
    <p v-if="catalog.error.value" role="alert" class="text-fg-danger">
      {{ getErrorMessage(catalog.error.value) }}
    </p>
    <label v-for="resource in resources" :key="resource.uri" class="flex items-center gap-2"
      ><UiCheckbox
        :disabled="disabled"
        :label="resource.name"
        :model-value="choices.resourceUris.includes(resource.uri)"
        @update:model-value="toggle('resourceUris', resource.uri)"
      />{{ resource.name }}</label
    >
    <template v-if="catalog.data.value">
      <McpResourcePreview
        v-for="uri in choices.resourceUris"
        :key="uri"
        :connection="connection"
        :catalog-digest="catalog.data.value!.digest"
        :selection="{ type: 'resource', uri }"
        :disabled="disabled"
      />
      <McpResourceTemplateForm
        v-for="template in catalog.data.value?.resourceTemplates"
        :key="template.uriTemplate"
        :uri-template="template.uriTemplate"
        :disabled="disabled"
        @select="addTemplate"
      />
      <section v-for="(template, index) in choices.resourceTemplates" :key="index" class="space-y-1">
        <p class="break-all">{{ template.uriTemplate }} · {{ JSON.stringify(template.parameters) }}</p>
        <UiButton size="sm" variant="ghost" :disabled="disabled" @click="removeTemplate(index)">{{
          $t("common.remove")
        }}</UiButton>
        <McpResourcePreview
          :connection="connection"
          :catalog-digest="catalog.data.value!.digest"
          :selection="{ type: 'template', selection: template }"
          :disabled="disabled"
        />
      </section>
    </template>
    <label v-for="name in prompts" :key="name" class="flex items-center gap-2"
      ><UiCheckbox
        :disabled="disabled"
        :label="name"
        :model-value="choices.promptNames.includes(name)"
        @update:model-value="toggle('promptNames', name)"
      />{{ name }}</label
    >
  </fieldset>
</template>
