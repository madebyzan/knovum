<script setup lang="ts">
import { useMutation } from "@tanstack/vue-query";
import { computed } from "vue";

import { getErrorMessage } from "../../../../foundation/errors/error-format";
import { checkMcpConnection } from "../../../../foundation/extensions/connection-operations";
import type { AgentConnectionSelection } from "../../../../shared/contracts/generated/AgentConnectionSelection";
import type { AgentExtensionSelection } from "../../../../shared/contracts/generated/AgentExtensionSelection";
import type { McpConnection } from "../../../../shared/contracts/generated/McpConnection";
import type { McpResourceTemplateSelection as TemplateSelection } from "../../../../shared/contracts/generated/McpResourceTemplateSelection";
import UiButton from "../../../../shared/ui/UiButton.vue";
import McpResourcePreview from "./McpResourcePreview.vue";
import McpResourceTemplateSelection from "./McpResourceTemplateSelection.vue";
const selection = defineModel<AgentExtensionSelection>({ required: true });
const props = defineProps<{ connection: McpConnection }>();
const choices = computed(() =>
  selection.value.connections.find((candidate) => candidate.connectionId === props.connection.id)!,
);
const availableTools = computed(() => props.connection.tools.filter((tool) => tool.policy !== "deny"));
const catalog = useMutation({ mutationFn: () => checkMcpConnection(props.connection.id) });
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
function addTemplate(template: TemplateSelection) {
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
  <section class="space-y-2 border-t border-stroke-divider pt-2 text-caption">
    <h4 class="font-medium">{{ connection.name }}</h4>
    <label v-for="tool in availableTools" :key="tool.name" class="flex items-center gap-2"
      ><input
        type="checkbox"
        :checked="choices.toolNames.includes(tool.name)"
        @change="toggle('toolNames', tool.name)"
      />{{ tool.name }}</label
    >
    <UiButton size="sm" variant="ghost" :disabled="catalog.isPending.value" @click="catalog.mutate()">{{
      $t("extensions.catalog.browse")
    }}</UiButton>
    <p v-if="catalog.error.value" role="alert" class="text-fg-danger">
      {{ getErrorMessage(catalog.error.value) }}
    </p>
    <label v-for="resource in resources" :key="resource.uri" class="flex items-center gap-2"
      ><input
        type="checkbox"
        :checked="choices.resourceUris.includes(resource.uri)"
        @change="toggle('resourceUris', resource.uri)"
      />{{ resource.name }}</label
    >
    <template v-if="catalog.data.value">
      <McpResourcePreview
        v-for="uri in choices.resourceUris"
        :key="uri"
        :connection="connection"
        :catalog-digest="catalog.data.value!.digest"
        :selection="{ type: 'resource', uri }"
      />
      <McpResourceTemplateSelection
        v-for="template in catalog.data.value?.resourceTemplates"
        :key="template.uriTemplate"
        :uri-template="template.uriTemplate"
        @select="addTemplate"
      />
      <section v-for="(template, index) in choices.resourceTemplates" :key="index" class="space-y-1">
        <p class="break-all">{{ template.uriTemplate }} · {{ JSON.stringify(template.parameters) }}</p>
        <UiButton size="sm" variant="ghost" @click="removeTemplate(index)">{{
          $t("common.remove")
        }}</UiButton>
        <McpResourcePreview
          :connection="connection"
          :catalog-digest="catalog.data.value!.digest"
          :selection="{ type: 'template', selection: template }"
        />
      </section>
    </template>
    <label v-for="name in prompts" :key="name" class="flex items-center gap-2"
      ><input
        type="checkbox"
        :checked="choices.promptNames.includes(name)"
        @change="toggle('promptNames', name)"
      />{{ name }}</label
    >
  </section>
</template>
