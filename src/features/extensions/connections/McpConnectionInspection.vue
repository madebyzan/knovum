<script setup lang="ts">
import { Server, Trash2 } from "@lucide/vue";
import { shallowRef } from "vue";
import { useI18n } from "vue-i18n";

import { getErrorMessage } from "../../../foundation/errors/error-format";
import { deleteMcpConnection } from "../../../foundation/extensions/connection-operations";
import { isApplicationLeaveCancelled } from "../../../shared/contracts/application-leave";
import type { ConnectorDefinition } from "../../../shared/contracts/generated/ConnectorDefinition";
import type { McpConnection } from "../../../shared/contracts/generated/McpConnection";
import { useSettingsLeaveContext } from "../../../shared/contracts/settings-leave";
import { confirmAction } from "../../../shared/ui/confirm-action";
import UiButton from "../../../shared/ui/UiButton.vue";
import ExtensionAvailability from "../components/ExtensionAvailability.vue";
import ExtensionInspectionHeader from "../components/ExtensionInspectionHeader.vue";
import type { ExtensionDestination } from "../extension-destination";
import { useExtensionChanges } from "../use-extension-changes";
import McpConnectionEnablement from "./McpConnectionEnablement.vue";
import McpConnectionSettings from "./McpConnectionSettings.vue";
const props = defineProps<{ connection: McpConnection; connector: ConnectorDefinition }>();
const emit = defineEmits<{ navigate: [destination: ExtensionDestination] }>();
const { t } = useI18n();
const leaveCoordinator = useSettingsLeaveContext();
const changes = useExtensionChanges();
const navigationError = shallowRef("");
async function prepareChange() {
  navigationError.value = "";
  try {
    await leaveCoordinator.requestLeave();
    return true;
  } catch (cause) {
    if (!isApplicationLeaveCancelled(cause)) navigationError.value = getErrorMessage(cause);
    return false;
  }
}
async function remove() {
  if (
    !(await confirmAction({
      title: t("extensions.connections.delete.title", { name: props.connection.name }),
      description: t("extensions.connections.delete.description"),
      confirmLabel: t("common.delete"),
      danger: true,
    }))
  )
    return;
  if (!(await prepareChange())) return;
  void changes
    .mutateAsync(async () => {
      await deleteMcpConnection(props.connection);
      emit("navigate", { tab: "mcp" });
    })
    .catch(() => {});
}
</script>
<template>
  <section class="space-y-5">
    <ExtensionInspectionHeader
      :name="connection.name"
      :description="connector.name"
      :icon="Server"
      @back="$emit('navigate', { tab: 'mcp' })"
    >
      <template #status
        ><ExtensionAvailability
          class="mt-2"
          kind="connections"
          :extension-id="connection.id"
          :enabled="connection.enabled"
          :plugin-id="connector.pluginId"
      /></template>
      <UiButton
        size="sm"
        variant="ghost"
        class="text-fg-danger"
        :disabled="changes.isPending.value"
        @click="remove"
        ><Trash2 class="size-3.5" />{{ $t("common.delete") }}</UiButton
      >
      <McpConnectionEnablement
        :connection="connection"
        :configured="Boolean(connector.endpoint)"
        :disabled="changes.isPending.value"
      />
    </ExtensionInspectionHeader>
    <UiButton
      size="sm"
      variant="ghost"
      @click="$emit('navigate', { tab: 'applications', id: connector.id })"
      >{{ $t("extensions.connections.serviceAndAccounts") }}</UiButton
    >
    <p v-if="!connector.endpoint" class="text-caption text-fg-warning">
      {{ $t("extensions.connections.configureServiceFirst") }}
    </p>
    <p v-if="navigationError" role="alert" class="text-body text-fg-danger">{{ navigationError }}</p>
    <p v-if="changes.error.value" role="alert" class="text-body text-fg-danger">
      {{ getErrorMessage(changes.error.value) }}
    </p>
    <McpConnectionSettings :key="connection.id" :connection="connection" :connector="connector" />
  </section>
</template>
