<script setup lang="ts">
import { computed, reactive, watch } from "vue";
import { useI18n } from "vue-i18n";

import type { AiModelCapability, AiModelDefinition } from "../../../foundation/ai/ai-models";
import UiButton from "../../../shared/ui/UiButton.vue";
import UiCheckbox from "../../../shared/ui/UiCheckbox.vue";
import UiDialog from "../../../shared/ui/UiDialog.vue";
import UiInput from "../../../shared/ui/UiInput.vue";

const open = defineModel<boolean>("open", { required: true });
const props = defineProps<{ allowedCapabilities: readonly AiModelCapability[] }>();
const emit = defineEmits<{ create: [model: AiModelDefinition] }>();
const { t } = useI18n();
const draft = reactive({ name: "", model: "", capabilities: [] as AiModelCapability[] });
const options = computed(() =>
  [
    ["text-generation", t("settings.ai.capabilities.text")],
    ["vision-understanding", t("settings.ai.capabilities.vision")],
    ["image-generation", t("settings.ai.capabilities.image")],
    ["video-generation", t("settings.ai.capabilities.video")],
  ].map(([value, label]) => ({ value: value as AiModelCapability, label: label! })),
);
const valid = computed(
  () => draft.name.trim().length > 0 && draft.model.trim().length > 0 && draft.capabilities.length > 0,
);

watch(open, (visible) => {
  if (!visible) return;
  draft.name = "";
  draft.model = "";
  draft.capabilities = [props.allowedCapabilities[0]!];
});

function setCapability(capability: AiModelCapability, enabled: boolean) {
  if (enabled) draft.capabilities = [...draft.capabilities, capability];
  else if (draft.capabilities.length > 1) {
    draft.capabilities = draft.capabilities.filter((value) => value !== capability);
  }
}

function createModel() {
  if (!valid.value) return;
  emit("create", {
    id: crypto.randomUUID(),
    name: draft.name.trim(),
    model: draft.model.trim(),
    capabilities: [...draft.capabilities],
  });
  open.value = false;
}
</script>

<template>
  <UiDialog
    v-model:open="open"
    :title="t('settings.ai.addModel')"
    :description="t('settings.ai.addModelDescription')"
  >
    <div class="grid gap-4">
      <label class="grid gap-1.5 text-caption font-medium text-foreground-secondary">
        {{ t("settings.ai.modelDisplayName") }}
        <UiInput v-model="draft.name" autofocus />
      </label>
      <label class="grid gap-1.5 text-caption font-medium text-foreground-secondary">
        {{ t("settings.ai.modelIdentifier") }}
        <UiInput v-model="draft.model" />
      </label>
      <fieldset class="grid gap-2 border-t border-divider pt-4">
        <legend class="mb-1 text-caption font-medium text-foreground-secondary">
          {{ t("settings.ai.modelCapabilities") }}
        </legend>
        <label
          v-for="capability in options"
          :key="capability.value"
          class="flex items-center gap-2 text-label text-foreground"
          :class="!props.allowedCapabilities.includes(capability.value) && 'opacity-45'"
        >
          <UiCheckbox
            :model-value="draft.capabilities.includes(capability.value)"
            :label="capability.label"
            :disabled="!props.allowedCapabilities.includes(capability.value)"
            @update:model-value="setCapability(capability.value, $event)"
          />
          {{ capability.label }}
        </label>
      </fieldset>
    </div>
    <template #footer>
      <UiButton variant="ghost" @click="open = false">{{ t("common.cancel") }}</UiButton>
      <UiButton variant="primary" :disabled="!valid" @click="createModel">
        {{ t("settings.ai.addModel") }}
      </UiButton>
    </template>
  </UiDialog>
</template>
