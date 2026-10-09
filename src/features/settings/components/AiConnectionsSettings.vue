<script setup lang="ts">
import { Plus } from "@lucide/vue";
import { computed, watch } from "vue";
import { useI18n } from "vue-i18n";

import { requireProvider } from "../../../foundation/ai/providers";
import type { AiConnectionSettings } from "../../../shared/contracts/generated/AiConnectionSettings";
import { routingCapabilities } from "../../../shared/contracts/generated/contract-values";
import type { ProviderDescriptor } from "../../../shared/contracts/generated/ProviderDescriptor";
import type { RoutingPurpose } from "../../../shared/contracts/generated/RoutingPurpose";
import UiButton from "../../../shared/ui/UiButton.vue";
import { useAiConnectionEditing } from "../use-ai-connection-editing";
import AiConnectionConfiguration from "./AiConnectionConfiguration.vue";
import AiConnectionEditor from "./AiConnectionEditor.vue";

const props = defineProps<{
  connections: readonly AiConnectionSettings[];
  protectedConnectionIds: readonly string[];
  protectedModelTargets: readonly string[];
  providers: readonly ProviderDescriptor[];
  createModelRequest?: { purpose: RoutingPurpose } | undefined;
}>();
const { t } = useI18n();
const {
  selectedId,
  dirty,
  busy,
  error,
  expandedId,
  editing,
  openConnection,
  requestClose,
  saveConnection,
  removeConnection,
} = useAiConnectionEditing({
  connections: () => props.connections,
  protectedConnectionIds: () => props.protectedConnectionIds,
  providersAvailable: () => Boolean(props.providers.length),
  t,
});
const requestedConnectionId = computed(() => {
  const purpose = props.createModelRequest?.purpose;
  return purpose
    ? props.connections.find(({ provider }) =>
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
  () => props.connections.map(({ id }) => id),
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
      <AiConnectionConfiguration
        v-for="connection in connections"
        :key="connection.id"
        :connection="connection"
        :provider="requireProvider(providers, connection.provider)"
        :providers="providers"
        :expanded="expandedId === connection.id"
        :busy="busy"
        :protected="protectedConnectionIds.includes(connection.id)"
        :protected-model-targets="protectedModelTargets"
        :create-model-request="requestedConnectionId === connection.id ? createModelRequest : undefined"
        @toggle="expandedId = expandedId === connection.id ? undefined : connection.id"
        @edit="openConnection(connection.id)"
        @remove="removeConnection(connection)"
      />
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
