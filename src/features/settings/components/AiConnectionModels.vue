<script setup lang="ts">
import { Pencil, Plus, Trash2 } from "@lucide/vue";
import { computed, shallowRef, watch } from "vue";
import { useI18n } from "vue-i18n";

import { useAiSettingsStore } from "../../../foundation/ai/ai-settings-store";
import { encodeExecutionTarget } from "../../../foundation/ai/execution-target-codec";
import { requireProvider, type ProviderDescriptor } from "../../../foundation/ai/providers";
import type {
  AiConnectionSettings,
  AiModelCapability,
  AiModelDefinition,
  AiRoutingPurpose,
} from "../../../foundation/ai/schema/models";
import { confirmAction } from "../../../shared/ui/confirm-action";
import UiButton from "../../../shared/ui/UiButton.vue";
import UiIconButton from "../../../shared/ui/UiIconButton.vue";
import AiModelEditor from "./AiModelEditor.vue";

const props = defineProps<{
  connection: AiConnectionSettings;
  providers: readonly ProviderDescriptor[];
  protectedModelTargets: readonly string[];
  createRequest?: { purpose: AiRoutingPurpose } | undefined;
}>();
const { t } = useI18n();
const store = useAiSettingsStore();
const selected = shallowRef<string | null>();
const dirty = shallowRef(false);
const busy = shallowRef(false);
const error = shallowRef("");
const editingModel = computed(() => props.connection.models.find(({ id }) => id === selected.value));
const dialogOpen = computed({
  get: () => selected.value !== undefined,
  set: (value: boolean) => {
    if (!value) void requestClose();
  },
});
const capabilityKey: Record<AiModelCapability, string> = {
  "text-generation": "text",
  "vision-understanding": "vision",
  "image-generation": "image",
  "video-generation": "video",
  embedding: "embedding",
};

function modelInUse(modelId: string) {
  return props.protectedModelTargets.includes(
    encodeExecutionTarget({ type: "model", connectionId: props.connection.id, modelId }),
  );
}

function openModel(modelId?: string) {
  if (busy.value) return;
  selected.value = modelId ?? null;
  dirty.value = false;
  error.value = "";
}
watch(
  () => props.createRequest,
  (request) => {
    if (request) openModel();
  },
);

function closeEditor() {
  selected.value = undefined;
  dirty.value = false;
  error.value = "";
}

async function requestClose() {
  if (busy.value) return;
  if (
    dirty.value &&
    !(await confirmAction({
      title: t("settings.ai.discardChangesTitle"),
      description: t("settings.ai.discardChanges"),
      confirmLabel: t("settings.ai.discardChangesAction"),
      danger: true,
    }))
  )
    return;
  closeEditor();
}

async function saveModel(model: AiModelDefinition) {
  const connection = props.connection;
  busy.value = true;
  error.value = "";
  const index = connection.models.findIndex(({ id }) => id === model.id);
  if (connection.models.some(({ id, model: identifier }) => id !== model.id && identifier === model.model)) {
    error.value = t("settings.ai.duplicateModelIdentifier");
    busy.value = false;
    return;
  }
  const previous = index >= 0 ? connection.models[index] : undefined;
  if (index >= 0) connection.models[index] = model;
  else connection.models.push(model);
  try {
    await store.flush();
    closeEditor();
  } catch (cause) {
    if (previous) connection.models[index] = previous;
    else {
      const addedIndex = connection.models.findIndex(({ id }) => id === model.id);
      if (addedIndex >= 0) connection.models.splice(addedIndex, 1);
    }
    error.value = `${t("settings.ai.integrationSaveFailed")}: ${String(cause)}`;
  } finally {
    busy.value = false;
  }
}

async function removeModel(model: AiModelDefinition) {
  const connection = props.connection;
  if (busy.value || modelInUse(model.id)) return;
  const confirmed = await confirmAction({
    title: t("common.delete"),
    description: t("settings.ai.deleteModelConfirm", { name: model.name }),
    confirmLabel: t("common.delete"),
    danger: true,
  });
  if (!confirmed) return;
  busy.value = true;
  error.value = "";
  const index = connection.models.findIndex(({ id }) => id === model.id);
  if (index < 0) {
    busy.value = false;
    return;
  }
  connection.models.splice(index, 1);
  try {
    await store.flush();
  } catch (cause) {
    connection.models.splice(index, 0, model);
    error.value = `${t("settings.ai.integrationSaveFailed")}: ${String(cause)}`;
  } finally {
    busy.value = false;
  }
}
</script>

<template>
  <div class="border-t border-stroke-divider px-3 pb-2">
    <div class="flex items-center justify-between gap-3 px-2 py-2.5">
      <p class="text-caption font-medium text-fg-secondary">{{ t("settings.ai.models") }}</p>
      <UiButton size="sm" variant="ghost" :disabled="busy" @click="openModel()"
        ><Plus class="size-3.5" />{{ t("settings.ai.addModel") }}</UiButton
      >
    </div>
    <p v-if="!connection.models.length" class="px-2 py-3 text-caption text-fg-secondary">
      {{ t("settings.ai.noModelsForConnection") }}
    </p>
    <div v-else class="border-t border-stroke-divider">
      <div
        v-for="model in connection.models"
        :key="model.id"
        class="flex min-h-14 items-center gap-3 border-b border-stroke-divider px-2 py-2 last:border-b-0"
      >
        <div class="min-w-0 flex-1">
          <p class="truncate text-label font-medium text-fg-primary">{{ model.name }}</p>
          <p v-if="model.model !== model.name" class="truncate text-caption text-fg-tertiary">
            {{ model.model }}
          </p>
          <p class="mt-0.5 truncate text-caption text-fg-secondary">
            {{
              model.capabilities
                .map((capability) => t(`settings.ai.capabilities.${capabilityKey[capability]}`))
                .join(" · ")
            }}
          </p>
        </div>
        <UiIconButton :label="t('common.edit')" size="sm" :disabled="busy" @click="openModel(model.id)"
          ><Pencil class="size-4"
        /></UiIconButton>
        <UiIconButton
          :label="t('common.delete')"
          size="sm"
          class="hover:text-fg-danger"
          :disabled="busy || modelInUse(model.id)"
          @click="removeModel(model)"
          ><Trash2 class="size-4"
        /></UiIconButton>
      </div>
    </div>
    <p v-if="error && selected === undefined" class="px-2 py-2 text-caption text-fg-danger" role="alert">
      {{ error }}
    </p>
    <AiModelEditor
      v-if="selected !== undefined"
      :key="selected ?? 'new'"
      v-model:open="dialogOpen"
      :connection-name="connection.name"
      :model="editingModel"
      :allowed-capabilities="requireProvider(providers, connection.provider).capabilities"
      :protected="Boolean(editingModel && modelInUse(editingModel.id))"
      :busy="busy"
      :error="error"
      @dirty="dirty = $event"
      @save="saveModel"
    />
  </div>
</template>
