<script setup lang="ts">
import { Pencil, Plus, Trash2 } from "@lucide/vue";
import { computed, watch } from "vue";
import { useI18n } from "vue-i18n";

import { requireProvider } from "../../../../../foundation/ai/providers";
import type { AiConnectionSettings } from "../../../../../shared/contracts/generated/AiConnectionSettings";
import type { ExecutionCapability } from "../../../../../shared/contracts/generated/ExecutionCapability";
import type { ProviderDescriptor } from "../../../../../shared/contracts/generated/ProviderDescriptor";
import type { RoutingPurpose } from "../../../../../shared/contracts/generated/RoutingPurpose";
import UiButton from "../../../../../shared/ui/UiButton.vue";
import UiIconButton from "../../../../../shared/ui/UiIconButton.vue";
import AiModelEditor from "../../models/components/AiModelEditor.vue";
import { useAiModelEditing } from "../../models/use-ai-model-editing";

const props = defineProps<{
  connection: AiConnectionSettings;
  providers: readonly ProviderDescriptor[];
  protectedModelTargets: readonly string[];
  createRequest?: { purpose: RoutingPurpose } | undefined;
}>();
const { t } = useI18n();
const {
  selected,
  dirty,
  busy,
  error,
  editingModel,
  modelInUse,
  openModel,
  requestClose,
  saveModel,
  removeModel,
} = useAiModelEditing({
  connection: () => props.connection,
  protectedModelTargets: () => props.protectedModelTargets,
});
const dialogOpen = computed({
  get: () => selected.value !== undefined,
  set: (value: boolean) => {
    if (!value) void requestClose();
  },
});
const LABEL_KEY_BY_CAPABILITY: Record<ExecutionCapability, string> = {
  "text-generation": "text",
  "vision-understanding": "vision",
  "image-generation": "image",
  "video-generation": "video",
  embedding: "embedding",
};

watch(
  () => props.createRequest,
  (request) => {
    if (request) openModel();
  },
);
</script>

<template>
  <div class="border-t border-stroke-divider px-3 pb-2">
    <div class="flex items-center justify-between gap-3 px-2 py-2.5">
      <p class="text-caption font-medium text-fg-secondary">{{ t("settings.ai.models.title") }}</p>
      <UiButton size="sm" variant="ghost" :disabled="busy" @click="openModel()"
        ><Plus class="size-3.5" />{{ t("settings.ai.models.add") }}</UiButton
      >
    </div>
    <p v-if="!connection.models.length" class="px-2 py-3 text-caption text-fg-secondary">
      {{ t("settings.ai.models.empty") }}
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
                .map((capability) =>
                  t(`settings.ai.models.capabilities.${LABEL_KEY_BY_CAPABILITY[capability]}`),
                )
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
      :builtin-agent-protocols="requireProvider(providers, connection.provider).builtinAgentProtocols"
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
