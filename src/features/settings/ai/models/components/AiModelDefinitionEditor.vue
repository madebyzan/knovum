<script setup lang="ts">
import { Check, Image, PenLine, ScanSearch, Video } from "@lucide/vue";
import { computed, shallowRef, type Component } from "vue";
import { useI18n } from "vue-i18n";

import type { AiModelDefinition } from "../../../../../shared/contracts/generated/AiModelDefinition";
import type { ExecutionCapability } from "../../../../../shared/contracts/generated/ExecutionCapability";
import UiInput from "../../../../../shared/ui/UiInput.vue";

const model = defineModel<AiModelDefinition>({ required: true });
const props = defineProps<{
  protected: boolean;
  allowedCapabilities: readonly ExecutionCapability[];
  showErrors?: boolean;
}>();
const { t } = useI18n();
const capabilityTouched = shallowRef(false);

const CAPABILITY_DEFINITIONS: readonly {
  value: ExecutionCapability;
  labelKey: string;
  icon: Component;
}[] = [
  { value: "text-generation", labelKey: "settings.ai.models.capabilities.text", icon: PenLine },
  { value: "vision-understanding", labelKey: "settings.ai.models.capabilities.vision", icon: ScanSearch },
  { value: "image-generation", labelKey: "settings.ai.models.capabilities.image", icon: Image },
  { value: "video-generation", labelKey: "settings.ai.models.capabilities.video", icon: Video },
];

const capabilities = computed(() =>
  CAPABILITY_DEFINITIONS.filter(({ value }) => props.allowedCapabilities.includes(value)).map((item) => ({
    value: item.value,
    label: t(item.labelKey),
    icon: item.icon,
  })),
);

const gridColsClass = computed(() => {
  if (capabilities.value.length === 1) return "grid-cols-1 sm:grid-cols-2";
  if (capabilities.value.length === 2) return "grid-cols-2";
  if (capabilities.value.length === 4) return "grid-cols-2 sm:grid-cols-4";
  return "grid-cols-3";
});

function toggleCapability(capability: ExecutionCapability) {
  const selected = model.value.capabilities.includes(capability);
  if (selected && props.protected) return;
  capabilityTouched.value = true;
  if (selected && capability === "text-generation") model.value.builtinAgent = false;
  model.value.capabilities = selected
    ? model.value.capabilities.filter((value) => value !== capability)
    : [...model.value.capabilities, capability];
}
</script>

<template>
  <div class="pt-2 pb-4">
    <div class="grid gap-4">
      <label class="grid min-w-0 gap-1 text-caption text-fg-secondary">
        {{ t("settings.ai.models.displayName") }}
        <UiInput v-model="model.name" />
        <span v-if="showErrors && !model.name.trim()" class="text-fg-danger">{{ t("common.required") }}</span>
      </label>
      <label class="grid min-w-0 gap-1 text-caption text-fg-secondary">
        {{ t("settings.ai.models.identifier") }}
        <UiInput v-model="model.model" />
        <span v-if="showErrors && !model.model.trim()" class="text-fg-danger">{{
          t("common.required")
        }}</span>
      </label>
    </div>
    <p class="mt-5 text-caption font-medium text-fg-secondary">
      {{ t("settings.ai.models.capabilities.label") }}
    </p>
    <div
      class="mt-2 grid gap-2.5"
      :class="gridColsClass"
      role="group"
      :aria-label="t('settings.ai.models.capabilities.label')"
    >
      <button
        v-for="capability in capabilities"
        :key="capability.value"
        type="button"
        :aria-pressed="model.capabilities.includes(capability.value)"
        :disabled="protected && model.capabilities.includes(capability.value)"
        class="group relative flex flex-col items-center justify-center gap-2 rounded-lg border px-2 py-3 text-center transition-all duration-150 select-none focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-stroke-focus disabled:cursor-not-allowed disabled:opacity-60"
        :class="
          model.capabilities.includes(capability.value)
            ? 'border-stroke-accent bg-surface-accent-subtle text-fg-accent shadow-xs'
            : 'border-stroke-default bg-surface-raised text-fg-secondary hover:border-stroke-default-hover hover:bg-surface-control-hover hover:text-fg-primary'
        "
        @click="toggleCapability(capability.value)"
      >
        <div
          class="flex size-8 items-center justify-center rounded-md transition-colors"
          :class="
            model.capabilities.includes(capability.value)
              ? 'bg-surface-accent/15 text-fg-accent'
              : 'bg-surface-control text-fg-tertiary group-hover:text-fg-secondary'
          "
        >
          <component :is="capability.icon" class="size-4 shrink-0" />
        </div>
        <span class="text-caption leading-snug font-medium">{{ capability.label }}</span>
        <span
          v-if="model.capabilities.includes(capability.value)"
          class="absolute top-1.5 right-1.5 flex size-4 items-center justify-center rounded-full bg-surface-accent text-fg-on-accent"
        >
          <Check class="size-2.5 stroke-[3]" />
        </span>
      </button>
    </div>
    <p v-if="protected" class="mt-2 text-caption text-fg-secondary">{{ t("settings.ai.models.inUse") }}</p>
    <p
      v-if="(showErrors || capabilityTouched) && !model.capabilities.length"
      class="mt-2 text-caption text-fg-danger"
      role="alert"
    >
      {{ t("settings.ai.models.capabilities.required") }}
    </p>
  </div>
</template>
