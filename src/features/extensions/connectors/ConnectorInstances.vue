<script setup lang="ts">
import { Plus } from "@lucide/vue";
import { useI18n } from "vue-i18n";

import { getErrorMessage } from "../../../foundation/errors/error-format";
import type { ConnectorDefinition } from "../../../shared/contracts/generated/ConnectorDefinition";
import type { McpConnection } from "../../../shared/contracts/generated/McpConnection";
import UiButton from "../../../shared/ui/UiButton.vue";
import UiInput from "../../../shared/ui/UiInput.vue";
import UiSwitch from "../../../shared/ui/UiSwitch.vue";
import McpConnectionSettings from "../connections/McpConnectionSettings.vue";
import { useConnectorInstances } from "./use-connector-instances";

const props = defineProps<{ connector: ConnectorDefinition; connections: McpConnection[]; busy: boolean }>();
const { t } = useI18n();
const {
  adding,
  changes,
  name,
  selectedId,
  selected,
  create,
  toggleAdding,
  toggleSelected,
  toggleEnabled,
  remove,
} = useConnectorInstances({ connector: () => props.connector, connections: () => props.connections, t });
</script>
<template>
  <div class="space-y-4">
    <section class="space-y-3">
      <header class="flex items-center justify-between gap-3">
        <h4 class="text-body font-medium">{{ $t("extensions.connections.title") }}</h4>
        <UiButton
          size="sm"
          variant="ghost"
          :disabled="busy || changes.isPending.value || !connector.endpoint"
          :aria-expanded="adding"
          data-testid="add-connection-instance"
          @click="toggleAdding"
          ><Plus class="size-3.5" />{{ $t("extensions.connections.add") }}</UiButton
        >
      </header>
      <form v-if="adding" class="space-y-2 rounded-lg bg-surface-muted p-3" @submit.prevent="create">
        <label class="block space-y-1.5 text-caption text-fg-secondary"
          >{{ $t("extensions.connections.name")
          }}<UiInput
            v-model="name"
            :disabled="busy || changes.isPending.value"
            data-testid="connection-instance-name"
        /></label>
        <div class="flex justify-end gap-2">
          <UiButton size="sm" variant="ghost" :disabled="changes.isPending.value" @click="toggleAdding">{{
            $t("common.cancel")
          }}</UiButton
          ><UiButton
            type="submit"
            variant="primary"
            size="sm"
            :disabled="!name.trim() || busy || changes.isPending.value"
            data-testid="create-connection-instance"
            >{{ $t("extensions.connections.add") }}</UiButton
          >
        </div>
      </form>
      <p v-if="!connections.length" class="rounded-md bg-surface-muted p-3 text-caption text-fg-secondary">
        {{ $t("extensions.connections.empty") }}
      </p>
      <div
        v-for="connection in connections"
        :key="connection.id"
        class="flex flex-wrap items-center gap-2 rounded-md bg-surface-muted px-3 py-2.5"
        :data-testid="`mcp-connection-${connection.id}`"
      >
        <span class="min-w-0 flex-1 truncate text-body font-medium">{{ connection.name }}</span>
        <UiSwitch
          :model-value="connection.enabled"
          :disabled="busy || changes.isPending.value"
          :label="
            $t(connection.enabled ? 'extensions.plugins.disable' : 'extensions.plugins.enable', {
              name: connection.name,
            })
          "
          @update:model-value="toggleEnabled(connection)"
        />
        <UiButton
          size="sm"
          variant="ghost"
          :aria-expanded="selectedId === connection.id"
          @click="toggleSelected(connection.id)"
          >{{ $t("extensions.connections.accountsAndTools") }}</UiButton
        >
        <UiButton
          size="sm"
          variant="ghost"
          class="text-fg-danger"
          :disabled="busy || changes.isPending.value"
          :data-testid="`delete-connection-${connection.id}`"
          @click="remove(connection)"
          >{{ $t("common.delete") }}</UiButton
        >
      </div>
    </section>
    <p v-if="changes.error.value" role="alert" class="text-caption text-fg-danger">
      {{ getErrorMessage(changes.error.value) }}
    </p>
    <section v-if="selected" class="space-y-3 rounded-lg bg-surface-muted p-3">
      <h4 class="text-body font-medium">
        {{ selected.name }} · {{ $t("extensions.connections.accountsAndTools") }}
      </h4>
      <McpConnectionSettings :key="selected.id" :connection="selected" :connector="connector" />
    </section>
  </div>
</template>
