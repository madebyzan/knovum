<script setup lang="ts">
import { Blocks, Plus } from "@lucide/vue";
import { useQuery } from "@tanstack/vue-query";
import { computed } from "vue";

import { getErrorMessage } from "../../../foundation/errors/error-format";
import {
  connectorQuery,
  connectionQuery,
  extensionAvailabilityQuery,
} from "../../../foundation/extensions/extension-query";
import UiButton from "../../../shared/ui/UiButton.vue";
import UiSpinner from "../../../shared/ui/UiSpinner.vue";
import ExtensionCleanupNotice from "../components/ExtensionCleanupNotice.vue";
import ExtensionEmptyState from "../components/ExtensionEmptyState.vue";
import ExtensionIdentity from "../components/ExtensionIdentity.vue";
import type { ExtensionDestination } from "../extension-destination";
import ConnectorConfiguration from "./ConnectorConfiguration.vue";
import ConnectorInspection from "./ConnectorInspection.vue";
import { useConnectorEditing } from "./use-connector-editing";
const props = withDefaults(defineProps<{ search?: string; selectedId?: string | undefined }>(), {
  search: "",
});
defineEmits<{ navigate: [destination: ExtensionDestination] }>();
const connectors = useQuery(connectorQuery);
const connections = useQuery(connectionQuery);
const availability = useQuery(extensionAvailabilityQuery);
const visible = computed(() =>
  (connectors.data.value ?? []).filter((connector) =>
    `${connector.name} ${connector.description}`.toLowerCase().includes(props.search.trim().toLowerCase()),
  ),
);
const selected = computed(() =>
  connectors.data.value?.find((connector) => connector.id === props.selectedId),
);
const availableCounts = computed(() => {
  const counts: Record<string, number> = {};
  for (const connection of availability.data.value?.connections ?? [])
    counts[connection.connectorId] = (counts[connection.connectorId] ?? 0) + 1;
  return counts;
});
const { changes, editing, draft, publicFields, edit, closeEditor, save, remove } = useConnectorEditing();
const error = computed(
  () => connectors.error.value || connections.error.value || availability.error.value || changes.error.value,
);
</script>
<template>
  <section class="min-w-0 space-y-4">
    <header v-if="!selectedId" class="flex justify-end">
      <UiButton :disabled="changes.isPending.value" data-testid="create-connector" @click="edit()"
        ><Plus class="size-4" />{{ $t("extensions.connectors.add") }}</UiButton
      >
    </header>
    <p v-if="error" role="alert" class="text-body text-fg-danger">{{ getErrorMessage(error) }}</p>
    <ExtensionCleanupNotice :errors="changes.data.value?.cleanupErrors ?? []" />
    <ConnectorInspection
      v-if="selected && connections.data.value"
      :connector="selected"
      :connections="connections.data.value"
      :busy="changes.isPending.value"
      @edit="edit(selected)"
      @remove="remove(selected)"
      @navigate="$emit('navigate', $event)"
    />
    <div
      v-else-if="connectors.isPending.value || connections.isPending.value"
      role="status"
      class="flex justify-center gap-2 py-12 text-body text-fg-secondary"
    >
      <UiSpinner class="size-4" />{{ $t("common.loading") }}
    </div>
    <ExtensionEmptyState
      v-else-if="selectedId && !error"
      :title="$t('extensions.overview.notFound')"
      :description="$t('extensions.overview.notFoundHint')"
      ><UiButton @click="$emit('navigate', { tab: 'applications' })">{{
        $t("extensions.overview.back")
      }}</UiButton></ExtensionEmptyState
    >
    <template v-else-if="!selectedId">
      <p class="text-caption text-fg-secondary">{{ $t("extensions.connectors.description") }}</p>
      <ExtensionEmptyState
        v-if="!connectors.error.value && !visible.length"
        :title="$t(search ? 'extensions.plugins.search.empty' : 'extensions.connectors.empty.title')"
        :description="
          $t(search ? 'extensions.plugins.search.hint' : 'extensions.connectors.empty.description')
        "
      />
      <div class="divide-y divide-stroke-divider">
        <ExtensionIdentity
          v-for="connector in visible"
          :key="connector.id"
          :name="connector.name"
          :description="connector.description"
          :icon="Blocks"
          :data-testid="`connector-${connector.id}`"
          @inspect="$emit('navigate', { tab: 'applications', id: connector.id })"
        >
          <template #status
            ><span v-if="availability.data.value" class="mt-1 block text-caption text-fg-tertiary">{{
              $t("extensions.connectors.availableCount", { count: availableCounts[connector.id] ?? 0 })
            }}</span></template
          >
          <UiButton
            size="sm"
            variant="ghost"
            :data-testid="`manage-connector-${connector.id}`"
            @click="$emit('navigate', { tab: 'applications', id: connector.id })"
            >{{ $t("extensions.connections.manage") }}</UiButton
          >
        </ExtensionIdentity>
      </div>
    </template>
    <ConnectorConfiguration
      v-model="draft"
      v-model:public-fields="publicFields"
      :open="editing"
      :busy="changes.isPending.value"
      :title="$t(draft.id ? 'extensions.connections.configure' : 'extensions.connectors.add')"
      :error="changes.error.value ? getErrorMessage(changes.error.value) : undefined"
      @save="save"
      @close="closeEditor"
    />
  </section>
</template>
