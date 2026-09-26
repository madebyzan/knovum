<script setup lang="ts">
import { Trash2 } from "@lucide/vue";
import { uniq } from "es-toolkit";
import { computed } from "vue";
import { useI18n } from "vue-i18n";

import type { AiModelCapability, AiModelDefinition } from "../../../foundation/ai/ai-models";
import UiCheckbox from "../../../shared/ui/UiCheckbox.vue";
import UiIconButton from "../../../shared/ui/UiIconButton.vue";
import UiInput from "../../../shared/ui/UiInput.vue";

const model = defineModel<AiModelDefinition>({ required: true });
const props = defineProps<{
  removable: boolean;
  protected: boolean;
  allowedCapabilities: readonly AiModelCapability[];
}>();
const emit = defineEmits<{ remove: [] }>();
const { t } = useI18n();
const capabilities = computed<{ value: AiModelCapability; label: string }[]>(() => [
  { value: "text-generation", label: t("settings.ai.capabilities.text") },
  { value: "vision-understanding", label: t("settings.ai.capabilities.vision") },
  { value: "image-generation", label: t("settings.ai.capabilities.image") },
  { value: "video-generation", label: t("settings.ai.capabilities.video") },
]);

function setCapability(capability: AiModelCapability, enabled: boolean) {
  if (!enabled && (props.protected || model.value.capabilities.length === 1)) return;
  model.value.capabilities = enabled
    ? uniq([...model.value.capabilities, capability])
    : model.value.capabilities.filter((value) => value !== capability);
}
</script>

<template>
  <div class="border-b border-divider px-3 py-3 last:border-b-0">
    <div class="grid grid-cols-[minmax(0,1fr)_minmax(0,1fr)_auto] items-center gap-2">
      <UiInput v-model="model.name" :placeholder="t('settings.ai.modelDisplayName')" />
      <UiInput v-model="model.model" :placeholder="t('settings.ai.modelIdentifier')" />
      <UiIconButton v-if="removable" :label="t('common.delete')" size="sm" @click="emit('remove')">
        <Trash2 class="size-(--knovum-icon-size-sm)" />
      </UiIconButton>
    </div>
    <div class="mt-2.5 flex flex-wrap gap-x-3 gap-y-1.5 px-0.5">
      <label
        v-for="capability in capabilities"
        :key="capability.value"
        class="flex items-center gap-1.5 text-caption text-foreground-secondary"
      >
        <UiCheckbox
          :model-value="model.capabilities.includes(capability.value)"
          :label="capability.label"
          :disabled="!props.allowedCapabilities.includes(capability.value)"
          @update:model-value="setCapability(capability.value, $event)"
        />
        {{ capability.label }}
      </label>
    </div>
  </div>
</template>
