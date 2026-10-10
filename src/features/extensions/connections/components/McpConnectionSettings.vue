<script setup lang="ts">
import { useMutation } from "@tanstack/vue-query";
import { cloneDeep, isEqual } from "es-toolkit";
import { computed, onScopeDispose, shallowRef, watch } from "vue";
import { useI18n } from "vue-i18n";

import { getErrorMessage } from "../../../../foundation/errors/error-format";
import {
  updateMcpConnection,
  checkMcpConnection,
} from "../../../../foundation/extensions/connection-operations";
import type { ConnectorDefinition } from "../../../../shared/contracts/generated/ConnectorDefinition";
import type { McpConnection } from "../../../../shared/contracts/generated/McpConnection";
import type { McpConnectionCatalog } from "../../../../shared/contracts/generated/McpConnectionCatalog";
import type { McpToolPermission } from "../../../../shared/contracts/generated/McpToolPermission";
import { useSettingsLeaveContext } from "../../../../shared/contracts/settings-leave";
import { confirmAction } from "../../../../shared/ui/confirm-action";
import UiButton from "../../../../shared/ui/UiButton.vue";
import UiInput from "../../../../shared/ui/UiInput.vue";
import { useExtensionChanges } from "../../use-extension-changes";
import ConnectionAccountSelection from "./ConnectionAccountSelection.vue";
import McpCatalogBrowser from "./McpCatalogBrowser.vue";
import McpToolSelection from "./McpToolSelection.vue";

const props = defineProps<{ connection: McpConnection; connector: ConnectorDefinition }>();
const { t } = useI18n();
const changes = useExtensionChanges();
const baseline = shallowRef(props.connection);
const hasConflict = shallowRef(false);
const name = shallowRef("");
const savedName = shallowRef("");
const catalog = shallowRef<McpConnectionCatalog>();
const permissions = shallowRef<McpToolPermission[]>([]);
const savedPermissions = shallowRef<McpToolPermission[]>([]);
const dirty = computed(
  () => name.value !== savedName.value || !isEqual(permissions.value, savedPermissions.value),
);
let admittedSave: Promise<boolean> | undefined;
const checking = useMutation({
  mutationFn: (connection: McpConnection) => checkMcpConnection(connection.id),
  onSuccess: (value, connection) => {
    if (connection.id !== props.connection.id || connection.revision !== props.connection.revision) return;
    catalog.value = value;
  },
});
watch(
  () => [props.connection.id, props.connection.revision],
  () => {
    if (baseline.value.id === props.connection.id && dirty.value) {
      hasConflict.value = baseline.value.revision !== props.connection.revision;
      return;
    }
    baseline.value = props.connection;
    hasConflict.value = false;
    name.value = props.connection.name;
    savedName.value = props.connection.name;
    catalog.value = undefined;
    permissions.value = cloneDeep(props.connection.tools);
    savedPermissions.value = cloneDeep(props.connection.tools);
    checking.reset();
  },
  { immediate: true },
);
function save() {
  const admitted: Promise<boolean> = changes
    .mutateAsync(async () => {
      const saved = await updateMcpConnection({
        ...baseline.value,
        name: name.value,
        tools: permissions.value,
      });
      baseline.value = saved;
      name.value = saved.name;
      savedName.value = saved.name;
      permissions.value = cloneDeep(saved.tools);
      savedPermissions.value = cloneDeep(saved.tools);
      hasConflict.value = false;
    })
    .then(
      () => true,
      () => false,
    )
    .finally(() => {
      if (admittedSave === admitted) admittedSave = undefined;
    });
  admittedSave = admitted;
}

async function prepareLeave() {
  if (admittedSave && !(await admittedSave)) return false;
  if (!dirty.value) return true;
  return confirmAction({
    title: t("extensions.connectors.discard.title"),
    description: t("extensions.connectors.discard.description"),
    confirmLabel: t("common.discardChanges"),
    danger: true,
  });
}
const unregister = useSettingsLeaveContext().register({
  prepare: prepareLeave,
  finalize() {
    name.value = savedName.value;
    permissions.value = cloneDeep(savedPermissions.value);
    if (hasConflict.value) {
      baseline.value = props.connection;
      name.value = props.connection.name;
      savedName.value = props.connection.name;
      permissions.value = cloneDeep(props.connection.tools);
      savedPermissions.value = cloneDeep(props.connection.tools);
      catalog.value = undefined;
      hasConflict.value = false;
    }
  },
});
onScopeDispose(unregister);
</script>

<template>
  <section class="space-y-3 border-t border-stroke-divider pt-3">
    <form v-if="!connector.pluginId" class="flex items-end gap-2" @submit.prevent="save">
      <label class="min-w-0 flex-1 text-caption"
        >{{ $t("extensions.connections.name")
        }}<UiInput
          v-model="name"
          :disabled="changes.isPending.value || checking.isPending.value || hasConflict"
      /></label>
      <UiButton
        type="submit"
        size="sm"
        :disabled="
          !name.trim() || !dirty || hasConflict || changes.isPending.value || checking.isPending.value
        "
        >{{ $t("common.save") }}</UiButton
      >
    </form>
    <ConnectionAccountSelection
      v-if="connector.authorization.type !== 'none'"
      :connection="connection"
      :disabled="dirty || checking.isPending.value"
    />
    <UiButton
      :disabled="
        checking.isPending.value ||
        changes.isPending.value ||
        dirty ||
        !connector.endpoint ||
        (connector.authorization.type !== 'none' && !connection.accountId)
      "
      data-testid="check-mcp-connection"
      @click="checking.mutate(connection)"
      >{{ $t("extensions.connections.check") }}</UiButton
    >
    <p v-if="changes.error.value || checking.error.value" role="alert" class="text-caption text-fg-danger">
      {{ getErrorMessage(changes.error.value || checking.error.value) }}
    </p>
    <p v-if="hasConflict" role="alert" class="text-caption text-fg-warning">
      {{ $t("extensions.connections.conflict") }}
    </p>
    <p v-else-if="dirty" class="text-caption text-fg-secondary">
      {{ $t("extensions.connections.saveBeforeCheck") }}
    </p>
    <McpCatalogBrowser
      v-if="catalog"
      :key="connection.revision"
      :connection="connection"
      :catalog="catalog"
    />
    <McpToolSelection
      v-if="catalog"
      v-model="permissions"
      :catalog="catalog"
      :can-save="dirty && Boolean(name.trim()) && !hasConflict"
      :busy="changes.isPending.value || checking.isPending.value || hasConflict"
      @save="save"
    />
  </section>
</template>
