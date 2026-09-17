<script setup lang="ts">
import { Plus } from "@lucide/vue";
import { ask } from "@tauri-apps/plugin-dialog";
import { shallowRef } from "vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";

import { aiConnectionSecretKey, type AiConnectionSettings } from "../../../foundation/ai/ai-models";
import type { ProviderDescriptor } from "../../../foundation/ai/providers";
import { deleteSecret } from "../../../foundation/secrets/secret-store";
import UiButton from "../../../shared/ui/UiButton.vue";
import AiConnectionCard from "./AiConnectionCard.vue";
import AiConnectionDialog from "./AiConnectionDialog.vue";
import SettingsSection from "./SettingsSection.vue";

const connections = defineModel<AiConnectionSettings[]>({ required: true });
const props = defineProps<{
  protectedConnectionIds: readonly string[];
  protectedModelTargets: readonly string[];
  providers: readonly ProviderDescriptor[];
}>();
const { t } = useI18n();
const createDialogOpen = shallowRef(false);

function addConnection(connection: AiConnectionSettings) {
  connections.value.push(connection);
}

async function removeConnection(index: number) {
  const connection = connections.value[index];
  if (!connection) return;
  const confirmed = await ask(t("settings.ai.deleteConnectionConfirm", { name: connection.name }), {
    title: t("common.delete"),
    kind: "warning",
    okLabel: t("common.delete"),
    cancelLabel: t("common.cancel"),
  });
  if (!confirmed) return;
  try {
    await deleteSecret(aiConnectionSecretKey(connection.id));
    connections.value.splice(index, 1);
  } catch (error) {
    toast.error(`${t("settings.ai.integrationSaveFailed")}: ${String(error)}`);
  }
}

function replaceConnection(index: number, connection: AiConnectionSettings) {
  connections.value[index] = connection;
}
</script>

<template>
  <SettingsSection
    id="settings-ai-providers"
    tabindex="-1"
    :title="t('settings.ai.connections')"
    :description="t('settings.ai.connectionsDescription')"
  >
    <div v-if="connections.length === 0" class="flex items-center justify-between gap-6 px-4 py-3">
      <div class="min-w-0">
        <p class="text-body-lg font-medium text-foreground">{{ t("settings.ai.noConnections") }}</p>
        <p class="mt-0.5 max-w-xl text-caption leading-normal text-foreground-secondary">
          {{ t("settings.ai.noConnectionsDescription") }}
        </p>
      </div>
      <UiButton class="shrink-0" @click="createDialogOpen = true">
        <Plus class="size-(--knovum-icon-size-sm)" />{{ t("settings.ai.addConnection") }}
      </UiButton>
    </div>
    <div v-else>
      <AiConnectionCard
        v-for="(connection, index) in connections"
        :key="connection.id"
        :model-value="connection"
        :providers="props.providers"
        :removable="!props.protectedConnectionIds.includes(connection.id)"
        :provider-locked="props.protectedConnectionIds.includes(connection.id)"
        :protected-model-targets="props.protectedModelTargets"
        @update:model-value="replaceConnection(index, $event)"
        @remove="removeConnection(index)"
      />
    </div>
    <div v-if="connections.length > 0" class="border-t border-divider px-3 py-2">
      <UiButton variant="ghost" @click="createDialogOpen = true">
        <Plus class="size-(--knovum-icon-size-sm)" />{{ t("settings.ai.addConnection") }}
      </UiButton>
    </div>
  </SettingsSection>
  <AiConnectionDialog v-model:open="createDialogOpen" :providers="props.providers" @create="addConnection" />
</template>
