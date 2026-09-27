<script setup lang="ts">
import { computed, reactive, watch } from "vue";
import { useI18n } from "vue-i18n";

import { requireProvider, type ProviderDescriptor } from "../../../foundation/ai/providers";
import {
  type AiModelCapability,
  type AiConnectionSettings,
  type AiProvider,
} from "../../../foundation/ai/schema/models";
import UiButton from "../../../shared/ui/UiButton.vue";
import UiCheckbox from "../../../shared/ui/UiCheckbox.vue";
import UiDialog from "../../../shared/ui/UiDialog.vue";
import UiInput from "../../../shared/ui/UiInput.vue";
import UiSelect from "../../../shared/ui/UiSelect.vue";
import { providerOptions } from "../ai-provider-options";

const open = defineModel<boolean>("open", { required: true });
const props = defineProps<{ providers: readonly ProviderDescriptor[] }>();
const emit = defineEmits<{ create: [connection: AiConnectionSettings] }>();
const { t } = useI18n();
const draft = reactive({
  name: "",
  provider: "openai" as AiProvider,
  baseUrl: "",
  modelName: "",
  modelId: "",
  capabilities: ["text-generation"] as AiModelCapability[],
});
const capabilityOptions = computed(() =>
  [
    ["text-generation", t("settings.ai.capabilities.text")],
    ["vision-understanding", t("settings.ai.capabilities.vision")],
    ["image-generation", t("settings.ai.capabilities.image")],
    ["video-generation", t("settings.ai.capabilities.video")],
  ].map(([value, label]) => ({ value: value as AiModelCapability, label: label! })),
);
const options = computed(() => providerOptions(props.providers));
const selectedProvider = computed(() => requireProvider(props.providers, draft.provider));
const valid = computed(
  () =>
    draft.name.trim().length > 0 &&
    draft.baseUrl.trim().length > 0 &&
    URL.canParse(draft.baseUrl.trim()) &&
    draft.modelName.trim().length > 0 &&
    draft.modelId.trim().length > 0 &&
    draft.capabilities.length > 0,
);

watch(open, (visible) => {
  if (!visible) return;
  Object.assign(draft, {
    name: "",
    provider: "openai",
    baseUrl: requireProvider(props.providers, "openai").defaultBaseUrl,
    modelName: "",
    modelId: "",
    capabilities: ["text-generation"],
  });
});

function selectProvider(provider: AiProvider) {
  draft.provider = provider;
  const descriptor = requireProvider(props.providers, provider);
  draft.baseUrl = descriptor.defaultBaseUrl;
  const allowed = descriptor.capabilities;
  draft.capabilities = draft.capabilities.filter((capability) => allowed.includes(capability));
  if (draft.capabilities.length === 0) draft.capabilities = [allowed[0]!];
}

function setCapability(capability: AiModelCapability, enabled: boolean) {
  if (enabled) draft.capabilities = [...draft.capabilities, capability];
  else if (draft.capabilities.length > 1) {
    draft.capabilities = draft.capabilities.filter((value) => value !== capability);
  }
}

function createConnection() {
  if (!valid.value) return;
  emit("create", {
    id: crypto.randomUUID(),
    name: draft.name.trim(),
    provider: draft.provider,
    baseUrl: draft.baseUrl.trim(),
    models: [
      {
        id: crypto.randomUUID(),
        name: draft.modelName.trim(),
        model: draft.modelId.trim(),
        capabilities: [...draft.capabilities],
      },
    ],
  });
  open.value = false;
}
</script>

<template>
  <UiDialog
    v-model:open="open"
    :title="t('settings.ai.addConnection')"
    :description="t('settings.ai.addConnectionDescription')"
  >
    <div class="grid gap-4">
      <label class="grid gap-1.5 text-caption font-medium text-foreground-secondary">
        {{ t("settings.ai.connectionName") }}
        <UiInput v-model="draft.name" autofocus />
      </label>
      <label class="grid gap-1.5 text-caption font-medium text-foreground-secondary">
        {{ t("settings.ai.provider") }}
        <UiSelect
          :model-value="draft.provider"
          :label="t('settings.ai.provider')"
          :options="options"
          @update:model-value="selectProvider($event as AiProvider)"
        />
      </label>
      <label class="grid gap-1.5 text-caption font-medium text-foreground-secondary">
        {{ t("settings.ai.baseUrl") }}
        <UiInput v-model="draft.baseUrl" />
      </label>
      <div class="grid grid-cols-2 gap-3 border-t border-divider pt-4">
        <label class="grid gap-1.5 text-caption font-medium text-foreground-secondary">
          {{ t("settings.ai.modelDisplayName") }}
          <UiInput v-model="draft.modelName" />
        </label>
        <label class="grid gap-1.5 text-caption font-medium text-foreground-secondary">
          {{ t("settings.ai.modelIdentifier") }}
          <UiInput v-model="draft.modelId" />
        </label>
      </div>
      <fieldset class="grid gap-2">
        <legend class="mb-1 text-caption font-medium text-foreground-secondary">
          {{ t("settings.ai.modelCapabilities") }}
        </legend>
        <label
          v-for="capability in capabilityOptions"
          :key="capability.value"
          class="flex items-center gap-2 text-label text-foreground"
          :class="!selectedProvider.capabilities.includes(capability.value) && 'opacity-45'"
        >
          <UiCheckbox
            :model-value="draft.capabilities.includes(capability.value)"
            :label="capability.label"
            :disabled="!selectedProvider.capabilities.includes(capability.value)"
            @update:model-value="setCapability(capability.value, $event)"
          />
          {{ capability.label }}
        </label>
      </fieldset>
    </div>
    <template #footer>
      <UiButton variant="ghost" @click="open = false">{{ t("common.cancel") }}</UiButton>
      <UiButton variant="primary" :disabled="!valid" @click="createConnection">
        {{ t("settings.ai.addConnection") }}
      </UiButton>
    </template>
  </UiDialog>
</template>
