<script setup lang="ts">
import { Plus } from "@lucide/vue";
import { useQuery } from "@tanstack/vue-query";
import { computed, shallowRef } from "vue";
import { useI18n } from "vue-i18n";

import { getErrorMessage } from "../../../foundation/errors/error-format";
import { connectorQuery, connectionQuery } from "../../../foundation/extensions/extension-query";
import UiButton from "../../../shared/ui/UiButton.vue";
import UiDialog from "../../../shared/ui/UiDialog.vue";
import UiSearchInput from "../../../shared/ui/UiSearchInput.vue";
import UiSettingsSection from "../../../shared/ui/UiSettingsSection.vue";
import UiSpinner from "../../../shared/ui/UiSpinner.vue";
import ExtensionCleanupNotice from "../components/ExtensionCleanupNotice.vue";
import ConnectorConnections from "./ConnectorConnections.vue";
import ConnectorEditor from "./ConnectorEditor.vue";
import { useConnectorEditing } from "./use-connector-editing";

const { t } = useI18n();
const search = shallowRef("");
const connectors = useQuery(connectorQuery);
const visible = computed(() =>
  (connectors.data.value ?? []).filter((connector) =>
    `${connector.name} ${connector.description}`.toLowerCase().includes(search.value.trim().toLowerCase()),
  ),
);
const connections = useQuery(connectionQuery);
const { changes, editing, draft, edit, closeEditor, save, remove } = useConnectorEditing(t);
</script>
<template>
  <section class="min-w-0 space-y-4">
    <header class="flex flex-wrap items-end justify-between gap-3 px-0.5">
      <div class="min-w-0 flex-1 basis-48">
        <h2 class="text-title-sm font-semibold tracking-title text-fg-primary">
          {{ $t("extensions.overview.connectors") }}
        </h2>
        <p class="mt-1 max-w-lg text-caption text-fg-secondary">
          {{ $t("extensions.connectors.description") }}
        </p>
      </div>
      <UiButton :disabled="changes.isPending.value" data-testid="create-connector" @click="edit()">
        <Plus class="size-4" />{{ $t("extensions.connectors.add") }}
      </UiButton>
    </header>
    <p
      v-if="connectors.error.value || connections.error.value || changes.error.value"
      role="alert"
      class="text-body text-fg-danger"
    >
      {{ getErrorMessage(connectors.error.value || connections.error.value || changes.error.value) }}
    </p>
    <ExtensionCleanupNotice :errors="changes.data.value?.cleanupErrors ?? []" />
    <UiSearchInput
      v-if="connectors.data.value?.length"
      v-model="search"
      :label="$t('extensions.connectors.search')"
      :placeholder="$t('extensions.connectors.search')"
      data-testid="connector-search"
    />
    <div
      v-if="connectors.isPending.value || connections.isPending.value"
      role="status"
      class="flex items-center justify-center gap-2 py-12 text-body text-fg-secondary"
    >
      <UiSpinner class="size-4" />{{ $t("common.loading") }}
    </div>
    <UiSettingsSection v-else-if="!connectors.error.value && !visible.length">
      <div class="px-5 py-6">
        <h3 class="text-body-lg font-medium text-fg-primary">
          {{ $t(search ? "extensions.plugins.search.empty" : "extensions.connectors.empty.title") }}
        </h3>
        <p class="mt-1 text-caption text-fg-secondary">
          {{ $t(search ? "extensions.plugins.search.hint" : "extensions.connectors.empty.description") }}
        </p>
      </div>
    </UiSettingsSection>
    <UiSettingsSection v-if="connections.data.value && visible.length">
      <div class="divide-y divide-stroke-divider">
        <ConnectorConnections
          v-for="connector in visible"
          :key="connector.id"
          :connector="connector"
          :connections="connections.data.value"
          :busy="changes.isPending.value"
          @edit="edit(connector)"
          @remove="remove(connector)"
        />
      </div>
    </UiSettingsSection>
    <p class="px-0.5 text-caption text-fg-tertiary">{{ $t("extensions.accounts.billingHint") }}</p>
    <UiDialog
      :open="editing"
      :title="$t(draft.id ? 'extensions.connections.configure' : 'extensions.connectors.add')"
      size="wide"
      :closable="!changes.isPending.value"
      @update:open="!$event && closeEditor()"
    >
      <ConnectorEditor
        v-if="editing"
        v-model="draft"
        :busy="changes.isPending.value"
        @save="save"
        @close="closeEditor"
      />
      <p v-if="changes.error.value" role="alert" class="mt-3 text-body text-fg-danger">
        {{ getErrorMessage(changes.error.value) }}
      </p>
    </UiDialog>
  </section>
</template>
