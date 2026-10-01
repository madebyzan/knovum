<script setup lang="ts">
import { ChevronDown, ChevronRight, Plus, Server, Settings, Trash2 } from "@lucide/vue";
import { computed, shallowRef, watch } from "vue";
import { useI18n } from "vue-i18n";

import { useAiSettingsStore } from "../../../foundation/ai/ai-settings-store";
import { requireProvider } from "../../../foundation/ai/providers";
import { aiConnectionSecretKey } from "../../../foundation/ai/schema/models";
import { deleteSecret, persistOptionalSecret } from "../../../foundation/secrets/secret-store";
import type { AiConnectionSettings } from "../../../shared/contracts/generated/AiConnectionSettings";
import { routingCapabilities } from "../../../shared/contracts/generated/contract-values";
import type { ProviderDescriptor } from "../../../shared/contracts/generated/ProviderDescriptor";
import type { RoutingPurpose } from "../../../shared/contracts/generated/RoutingPurpose";
import { confirmAction } from "../../../shared/ui/confirm-action";
import UiButton from "../../../shared/ui/UiButton.vue";
import UiIconButton from "../../../shared/ui/UiIconButton.vue";
import type { AiConnectionSubmission } from "../ai-connection-draft";
import { providerLogos } from "../ai-provider-logos";
import AiConnectionEditor from "./AiConnectionEditor.vue";
import AiConnectionModels from "./AiConnectionModels.vue";

const connections = defineModel<AiConnectionSettings[]>({ required: true });
const props = defineProps<{
  protectedConnectionIds: readonly string[];
  protectedModelTargets: readonly string[];
  providers: readonly ProviderDescriptor[];
  createModelRequest?: { purpose: RoutingPurpose } | undefined;
}>();
const { t } = useI18n();
const store = useAiSettingsStore();
const selectedId = shallowRef<string | null>();
const dirty = shallowRef(false);
const busy = shallowRef(false);
const error = shallowRef("");
const expandedId = shallowRef<string>();
const providerName = (connection: AiConnectionSettings) =>
  requireProvider(props.providers, connection.provider).displayName;
const editing = computed(() => connections.value.find((connection) => connection.id === selectedId.value));
const requestedConnectionId = computed(() => {
  const purpose = props.createModelRequest?.purpose;
  return purpose
    ? connections.value.find(({ provider }) =>
        requireProvider(props.providers, provider).capabilities.includes(routingCapabilities[purpose]),
      )?.id
    : undefined;
});
const dialogOpen = computed({
  get: () => selectedId.value !== undefined,
  set: (value: boolean) => {
    if (!value) void requestClose();
  },
});
watch(
  () => connections.value.map(({ id }) => id),
  (ids) => {
    if (!expandedId.value || !ids.includes(expandedId.value)) expandedId.value = ids[0];
  },
  { immediate: true },
);

watch(
  () => props.createModelRequest,
  (request) => {
    if (!request) return;
    if (requestedConnectionId.value) expandedId.value = requestedConnectionId.value;
    else openConnection(null);
  },
);

async function confirmDiscard() {
  return (
    !dirty.value ||
    confirmAction({
      title: t("settings.ai.discardChangesTitle"),
      description: t("settings.ai.discardChanges"),
      confirmLabel: t("settings.ai.discardChangesAction"),
      danger: true,
    })
  );
}

function openConnection(id: string | null) {
  if (busy.value || (id === null && !props.providers.length)) return;
  selectedId.value = id;
  dirty.value = false;
  error.value = "";
}

function closeEditor() {
  selectedId.value = undefined;
  dirty.value = false;
  error.value = "";
}

async function requestClose() {
  if (!busy.value && (await confirmDiscard())) closeEditor();
}

function rollbackConnection(index: number, previous: AiConnectionSettings | undefined, id: string) {
  if (previous) {
    connections.value[index] = previous;
    return;
  }
  const addedIndex = connections.value.findIndex((item) => item.id === id);
  if (addedIndex >= 0) connections.value.splice(addedIndex, 1);
}

async function saveConnection(submission: AiConnectionSubmission) {
  if (busy.value) return;
  busy.value = true;
  error.value = "";
  const { connection, secret } = submission;
  const index = connections.value.findIndex((item) => item.id === connection.id);
  const previous = index >= 0 ? connections.value[index] : undefined;
  let connectionSaved = false;
  if (index >= 0) connections.value[index] = connection;
  else connections.value.push(connection);
  try {
    await store.flush();
    connectionSaved = true;
    if (secret === null) await deleteSecret(aiConnectionSecretKey(connection.id));
    else if (secret !== undefined) await persistOptionalSecret(aiConnectionSecretKey(connection.id), secret);
    expandedId.value = connection.id;
    closeEditor();
  } catch (cause) {
    if (!connectionSaved) rollbackConnection(index, previous, connection.id);
    error.value = `${t(connectionSaved ? "settings.ai.secretSaveFailed" : "settings.ai.integrationSaveFailed")}: ${String(cause)}`;
  } finally {
    busy.value = false;
  }
}

async function removeConnection(connection: AiConnectionSettings) {
  if (busy.value || props.protectedConnectionIds.includes(connection.id)) return;
  const confirmed = await confirmAction({
    title: t("common.delete"),
    description: t("settings.ai.deleteConnectionConfirm", { name: connection.name }),
    confirmLabel: t("common.delete"),
    danger: true,
  });
  if (!confirmed) return;
  busy.value = true;
  error.value = "";
  const index = connections.value.findIndex((item) => item.id === connection.id);
  if (index < 0) {
    busy.value = false;
    return;
  }
  connections.value.splice(index, 1);
  try {
    await store.flush();
    await deleteSecret(aiConnectionSecretKey(connection.id));
    if (selectedId.value === connection.id) closeEditor();
  } catch (cause) {
    connections.value.splice(index, 0, connection);
    await store.flush();
    error.value = `${t("settings.ai.integrationSaveFailed")}: ${String(cause)}`;
  } finally {
    busy.value = false;
  }
}
</script>

<template>
  <section id="settings-ai-providers" tabindex="-1" class="min-w-0 outline-none">
    <header class="mb-3 flex items-end justify-between gap-4 px-0.5">
      <div>
        <h2 class="text-title-sm font-semibold tracking-title text-fg-primary">
          {{ t("settings.ai.connections") }}
        </h2>
        <p class="mt-1 text-caption text-fg-secondary">{{ t("settings.ai.connectionsDescription") }}</p>
      </div>
      <UiButton
        id="settings-ai-create-trigger"
        variant="secondary"
        :disabled="busy || !providers.length"
        @click="openConnection(null)"
        ><Plus class="size-4" />{{ t("settings.ai.addConnection") }}</UiButton
      >
    </header>
    <div v-if="connections.length" class="grid gap-2.5">
      <div
        v-for="connection in connections"
        :key="connection.id"
        class="overflow-hidden rounded-panel border border-stroke-default bg-surface-raised"
      >
        <div class="flex min-h-16 items-center gap-3 px-4 py-3 sm:px-5">
          <button
            type="button"
            class="flex min-w-0 flex-1 items-center gap-3 rounded-md text-left focus-visible:outline-2 focus-visible:outline-stroke-focus"
            :aria-expanded="expandedId === connection.id"
            :aria-controls="`connection-models-${connection.id}`"
            @click="expandedId = expandedId === connection.id ? undefined : connection.id"
          >
            <span
              aria-hidden="true"
              class="flex size-10 shrink-0 items-center justify-center rounded-lg bg-surface-panel"
            >
              <img
                v-if="providerLogos[connection.provider]"
                :src="providerLogos[connection.provider]?.src"
                alt=""
                class="size-5 object-contain"
                :class="providerLogos[connection.provider]?.monochrome && 'dark:invert'"
              />
              <Server v-else class="size-5 text-fg-secondary" />
            </span>
            <span class="min-w-0 flex-1"
              ><span class="block truncate text-body-lg font-semibold text-fg-primary">{{
                providerName(connection)
              }}</span
              ><span class="block truncate text-caption text-fg-secondary"
                >{{ connection.name }} ·
                {{ t("settings.ai.modelCount", { count: connection.models.length }) }}</span
              ></span
            >
            <ChevronDown
              v-if="expandedId === connection.id"
              aria-hidden="true"
              class="size-4 shrink-0 text-fg-secondary"
            />
            <ChevronRight v-else aria-hidden="true" class="size-4 shrink-0 text-fg-secondary" />
          </button>
          <span aria-hidden="true" class="mx-1 h-5 w-px bg-stroke-divider" />
          <UiIconButton
            :label="t('common.settings')"
            size="sm"
            :disabled="busy"
            @click="openConnection(connection.id)"
            ><Settings class="size-4"
          /></UiIconButton>
          <UiIconButton
            :label="t('settings.ai.deleteConnection')"
            size="sm"
            class="hover:text-fg-danger"
            :disabled="busy || protectedConnectionIds.includes(connection.id)"
            @click="removeConnection(connection)"
            ><Trash2 class="size-4"
          /></UiIconButton>
        </div>
        <div
          :id="`connection-models-${connection.id}`"
          class="grid transition-[grid-template-rows] duration-200 ease-standard motion-reduce:transition-none"
          :class="expandedId === connection.id ? 'grid-rows-[1fr]' : 'grid-rows-[0fr]'"
          :inert="expandedId !== connection.id"
          :aria-hidden="expandedId !== connection.id"
        >
          <div class="min-h-0 overflow-hidden">
            <AiConnectionModels
              :connection="connection"
              :providers="providers"
              :protected-model-targets="protectedModelTargets"
              :create-request="requestedConnectionId === connection.id ? createModelRequest : undefined"
            />
          </div>
        </div>
      </div>
    </div>
    <p
      v-else
      class="rounded-panel border border-stroke-default bg-surface-raised px-5 py-6 text-caption text-fg-secondary"
    >
      {{ t("settings.ai.noConnectionsDescription") }}
    </p>
    <p v-if="error && selectedId === undefined" class="px-1 py-2 text-caption text-fg-danger" role="alert">
      {{ error }}
    </p>
    <AiConnectionEditor
      v-if="selectedId === null || editing"
      :key="selectedId ?? 'new'"
      v-model:open="dialogOpen"
      :connection="editing"
      :providers="providers"
      :busy="busy"
      :error="error"
      @dirty="dirty = $event"
      @save="saveConnection"
    />
  </section>
</template>
