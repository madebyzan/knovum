<script setup lang="ts">
import { useQuery } from "@tanstack/vue-query";
import { cloneDeep } from "es-toolkit";
import { ref, shallowRef } from "vue";

import { getErrorMessage } from "../../../foundation/errors/error-format";
import {
  createConnectorDefinition,
  updateConnectorDefinition,
  deleteConnectorDefinition,
} from "../../../foundation/extensions/connector-operations";
import { connectorQuery, connectionQuery } from "../../../foundation/extensions/extension-query";
import type { ConnectorDefinition } from "../../../shared/contracts/generated/ConnectorDefinition";
import UiButton from "../../../shared/ui/UiButton.vue";
import ExtensionCleanupNotice from "../ExtensionCleanupNotice.vue";
import { useExtensionChanges } from "../use-extension-changes";
import { connectorDraft } from "./connector-draft";
import ConnectorCard from "./ConnectorCard.vue";
import ConnectorEditor from "./ConnectorEditor.vue";

const connectors = useQuery(connectorQuery);
const connections = useQuery(connectionQuery);
const changes = useExtensionChanges();
const editing = shallowRef(false);
const draft = ref(connectorDraft());
function edit(connector?: ConnectorDefinition) {
  draft.value = connector
    ? { ...cloneDeep(connector), endpoint: cloneDeep(connector.endpoint!) }
    : connectorDraft();
  editing.value = true;
}
function save() {
  void changes
    .mutateAsync(async () => {
      const result = draft.value.id
        ? await updateConnectorDefinition(draft.value)
        : await createConnectorDefinition(draft.value);
      editing.value = false;
      return result;
    })
    .catch(() => {});
}
function remove(connector: ConnectorDefinition) {
  void changes.mutateAsync(() => deleteConnectorDefinition(connector)).catch(() => {});
}
</script>

<template>
  <section class="space-y-4">
    <p class="text-body text-fg-secondary">{{ $t("extensions.connectorPurpose") }}</p>
    <UiButton :disabled="changes.isPending.value" data-testid="create-connector" @click="edit()">{{
      $t("extensions.addConnector")
    }}</UiButton>
    <p class="text-caption text-fg-secondary">{{ $t("extensions.externalServiceBilling") }}</p>
    <p
      v-if="connectors.error.value || connections.error.value || changes.error.value"
      role="alert"
      class="text-body text-fg-danger"
    >
      {{ getErrorMessage(connectors.error.value || connections.error.value || changes.error.value) }}
    </p>
    <ExtensionCleanupNotice :errors="changes.data.value?.cleanupErrors ?? []" />
    <ConnectorEditor
      v-if="editing"
      v-model="draft"
      :busy="changes.isPending.value"
      @save="save"
      @close="editing = false"
    />
    <p
      v-if="connectors.isPending.value || connections.isPending.value"
      class="text-caption text-fg-secondary"
    >
      {{ $t("extensions.loading") }}
    </p>
    <p
      v-else-if="!connectors.error.value && !connectors.data.value?.length"
      class="text-caption text-fg-secondary"
    >
      {{ $t("extensions.emptyConnectors") }}
    </p>
    <template v-if="connections.data.value">
      <ConnectorCard
        v-for="connector in connectors.data.value"
        :key="connector.id"
        :connector="connector"
        :connections="connections.data.value"
        :busy="changes.isPending.value"
        @edit="edit(connector)"
        @remove="remove(connector)"
      />
    </template>
  </section>
</template>
