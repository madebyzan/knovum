<script setup lang="ts">
import { Check } from "@lucide/vue";
import { computed, shallowRef } from "vue";
import { useI18n } from "vue-i18n";

import type { AiModelCapability, AiModelDefinition } from "../../../foundation/ai/schema/models";
import UiInput from "../../../shared/ui/UiInput.vue";

const model = defineModel<AiModelDefinition>({ required: true });
const props = defineProps<{
  protected: boolean;
  allowedCapabilities: readonly AiModelCapability[];
  showErrors?: boolean;
}>();
const { t } = useI18n();
const capabilityTouched = shallowRef(false);
const capabilities = computed(() =>
  (
    [
      { value: "text-generation", label: t("settings.ai.capabilities.text") },
      { value: "vision-understanding", label: t("settings.ai.capabilities.vision") },
      { value: "image-generation", label: t("settings.ai.capabilities.image") },
      { value: "video-generation", label: t("settings.ai.capabilities.video") },
    ] satisfies { value: AiModelCapability; label: string }[]
  ).filter(({ value }) => props.allowedCapabilities.includes(value)),
);

function toggleCapability(capability: AiModelCapability) {
  const selected = model.value.capabilities.includes(capability);
  if (selected && props.protected) return;
  capabilityTouched.value = true;
  model.value.capabilities = selected
    ? model.value.capabilities.filter((value) => value !== capability)
    : [...model.value.capabilities, capability];
}
</script>

<template>
  <div class="pt-2 pb-4">
    <div class="grid gap-4">
      <label class="grid min-w-0 gap-1 text-caption text-fg-secondary">
        {{ t("settings.ai.modelDisplayName") }}
        <UiInput v-model="model.name" />
        <span v-if="showErrors && !model.name.trim()" class="text-fg-danger">{{
          t("settings.ai.requiredField")
        }}</span>
      </label>
      <label class="grid min-w-0 gap-1 text-caption text-fg-secondary">
        {{ t("settings.ai.modelIdentifier") }}
        <UiInput v-model="model.model" />
        <span v-if="showErrors && !model.model.trim()" class="text-fg-danger">{{
          t("settings.ai.requiredField")
        }}</span>
      </label>
    </div>
    <p class="mt-5 text-caption font-medium text-fg-secondary">
      {{ t("settings.ai.modelCapabilitiesHelp") }}
    </p>
    <div class="mt-2 flex flex-wrap gap-2" role="group" :aria-label="t('settings.ai.modelCapabilitiesHelp')">
      <button
        v-for="capability in capabilities"
        :key="capability.value"
        type="button"
        :aria-pressed="model.capabilities.includes(capability.value)"
        :disabled="protected && model.capabilities.includes(capability.value)"
        class="inline-flex min-h-interaction items-center gap-1.5 rounded-md border px-3 py-1.5 text-label font-medium transition-colors duration-100 ease-standard focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-stroke-focus"
        :class="
          model.capabilities.includes(capability.value)
            ? 'border-stroke-accent bg-surface-accent-subtle text-fg-accent enabled:hover:bg-surface-selected'
            : 'border-stroke-default bg-surface-raised text-fg-secondary hover:border-stroke-default-hover hover:bg-surface-control-hover hover:text-fg-primary'
        "
        @click="toggleCapability(capability.value)"
      >
        <Check
          aria-hidden="true"
          class="size-3.5 shrink-0"
          :class="model.capabilities.includes(capability.value) ? 'opacity-100' : 'opacity-0'"
        />
        <span>{{ capability.label }}</span>
      </button>
    </div>
    <p v-if="protected" class="mt-2 text-caption text-fg-secondary">{{ t("settings.ai.modelInUse") }}</p>
    <p
      v-if="(showErrors || capabilityTouched) && !model.capabilities.length"
      class="mt-2 text-caption text-fg-danger"
      role="alert"
    >
      {{ t("settings.ai.selectCapability") }}
    </p>
  </div>
</template>
