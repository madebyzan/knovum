<script setup lang="ts">
import { ChevronRight, Plus, Trash2 } from "@lucide/vue";
import { intersection } from "es-toolkit";
import { CollapsibleContent, CollapsibleRoot, CollapsibleTrigger } from "reka-ui";
import { computed, reactive, shallowRef, watch } from "vue";
import { useI18n } from "vue-i18n";

import { encodeExecutionTarget } from "../../../foundation/ai/execution-target-codec";
import type { AiProvider } from "../../../foundation/ai/integration-settings";
import { requireProvider, type ProviderDescriptor } from "../../../foundation/ai/providers";
import { aiConnectionSecretKey, type AiConnectionSettings } from "../../../foundation/ai/schema/models";
import UiButton from "../../../shared/ui/UiButton.vue";
import UiIconButton from "../../../shared/ui/UiIconButton.vue";
import UiInput from "../../../shared/ui/UiInput.vue";
import UiSelect from "../../../shared/ui/UiSelect.vue";
import { createAiConnectionDraft, finalizeAiConnectionDraft } from "../ai-connection-draft";
import { providerOptions } from "../ai-provider-options";
import AiModelDefinitionEditor from "./AiModelDefinitionEditor.vue";
import AiModelDialog from "./AiModelDialog.vue";
import SecretPreferenceField from "./SecretPreferenceField.vue";

const connection = defineModel<AiConnectionSettings>({ required: true });
const props = withDefaults(
  defineProps<{
    removable?: boolean;
    providerLocked?: boolean;
    protectedModelTargets?: readonly string[];
    providers: readonly ProviderDescriptor[];
  }>(),
  { removable: true, providerLocked: false, protectedModelTargets: () => [] },
);
const emit = defineEmits<{ remove: [] }>();
const { t } = useI18n();
const open = shallowRef(false);
const modelDialogOpen = shallowRef(false);
const draft = reactive<AiConnectionSettings>(createAiConnectionDraft(connection.value));
const draftValid = computed(
  () => draft.name.trim().length > 0 && draft.models.length > 0 && URL.canParse(draft.baseUrl.trim()),
);
const options = computed(() => providerOptions(props.providers));
const selectedProvider = computed(() => requireProvider(props.providers, draft.provider));
const connectionProvider = computed(() => requireProvider(props.providers, connection.value.provider));

watch(open, (expanded) => {
  if (expanded) Object.assign(draft, createAiConnectionDraft(connection.value));
});

function selectProvider(provider: AiProvider) {
  draft.provider = provider;
  const descriptor = requireProvider(props.providers, provider);
  draft.baseUrl = descriptor.defaultBaseUrl;
  const allowed = descriptor.capabilities;
  for (const model of draft.models) {
    model.capabilities = intersection(model.capabilities, allowed);
    if (model.capabilities.length === 0) model.capabilities = [allowed[0]!];
  }
}

function addModel() {
  modelDialogOpen.value = true;
}

function replaceModel(index: number, model: AiConnectionSettings["models"][number]) {
  draft.models[index] = model;
}

function createModel(model: AiConnectionSettings["models"][number]) {
  draft.models.push(model);
}

function saveChanges() {
  connection.value = finalizeAiConnectionDraft(draft);
  open.value = false;
}
function isModelInUse(modelId: string) {
  return props.protectedModelTargets.includes(
    encodeExecutionTarget({
      type: "model",
      connectionId: connection.value.id,
      modelId,
    }),
  );
}
</script>

<template>
  <CollapsibleRoot v-model:open="open" class="border-b border-stroke-divider last:border-b-0">
    <div class="flex min-h-(--knovum-settings-row-min-height) items-center gap-3 px-4 py-3">
      <CollapsibleTrigger
        class="flex min-w-0 flex-1 items-center gap-3 rounded-md text-left outline-none focus-visible:ring-2 focus-visible:ring-stroke-focus"
      >
        <ChevronRight
          class="size-3.5 shrink-0 text-fg-tertiary transition-transform"
          :class="open && 'rotate-90'"
        />
        <div class="min-w-0 flex-1">
          <p class="truncate text-body-md font-medium text-fg-primary">{{ connection.name }}</p>
          <p class="mt-0.5 truncate text-caption-md text-fg-secondary">
            {{ connectionProvider.displayName }} ·
            {{ t("settings.ai.modelCount", { count: connection.models.length }) }}
          </p>
        </div>
      </CollapsibleTrigger>
      <UiIconButton v-if="props.removable" :label="t('common.delete')" size="sm" @click="emit('remove')">
        <Trash2 class="size-3.5" />
      </UiIconButton>
    </div>
    <CollapsibleContent class="border-t border-stroke-divider bg-surface-canvas/45 px-4 py-4">
      <div class="grid grid-cols-[minmax(0,1fr)_minmax(12rem,0.65fr)] items-start gap-3">
        <UiInput v-model="draft.name" :placeholder="t('settings.ai.connectionName')" />
        <UiSelect
          :model-value="draft.provider"
          :label="t('settings.ai.provider')"
          :options="options"
          :disabled="props.providerLocked"
          @update:model-value="selectProvider($event as AiProvider)"
        />
      </div>
      <UiInput v-model="draft.baseUrl" class="mt-3" :placeholder="t('settings.ai.baseUrl')" />
      <SecretPreferenceField
        :label="t('settings.ai.apiKey')"
        :secret-key="aiConnectionSecretKey(connection.id)"
      />
      <div class="mt-4">
        <div class="mb-2 flex items-center justify-between px-0.5">
          <p class="text-caption-md font-medium text-fg-secondary">{{ t("settings.ai.models") }}</p>
          <UiButton size="sm" variant="ghost" @click="addModel">
            <Plus class="size-3" />{{ t("settings.ai.addModel") }}
          </UiButton>
        </div>
        <div class="overflow-hidden rounded-md border border-stroke-default bg-surface-raised">
          <AiModelDefinitionEditor
            v-for="(model, index) in draft.models"
            :key="model.id"
            :model-value="model"
            :removable="!isModelInUse(model.id)"
            :protected="isModelInUse(model.id)"
            :allowed-capabilities="selectedProvider.capabilities"
            @update:model-value="replaceModel(index, $event)"
            @remove="draft.models.splice(index, 1)"
          />
        </div>
      </div>
      <div class="mt-4 flex justify-end gap-2 border-t border-stroke-divider pt-3">
        <UiButton variant="ghost" @click="open = false">{{ t("common.cancel") }}</UiButton>
        <UiButton variant="primary" :disabled="!draftValid" @click="saveChanges">
          {{ t("common.save") }}
        </UiButton>
      </div>
    </CollapsibleContent>
  </CollapsibleRoot>
  <AiModelDialog
    v-model:open="modelDialogOpen"
    :allowed-capabilities="selectedProvider.capabilities"
    @create="createModel"
  />
</template>
