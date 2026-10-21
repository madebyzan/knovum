<script setup lang="ts">
import { useId } from "reka-ui";
import { computed } from "vue";
import { useI18n } from "vue-i18n";

import { builtinAgentLimitsValid } from "../../../../../foundation/ai/schema/models";
import type { BuiltinAgentModelSettings } from "../../../../../shared/contracts/generated/BuiltinAgentModelSettings";
import UiInput from "../../../../../shared/ui/UiInput.vue";
import UiSwitch from "../../../../../shared/ui/UiSwitch.vue";

const model = defineModel<BuiltinAgentModelSettings | null>({ required: true });
defineProps<{ disabled: boolean; showErrors: boolean }>();
const { t } = useI18n();
const switchId = useId();
const enabled = computed({
  get: () => model.value !== null,
  set: (value: boolean) => {
    model.value = value ? { contextWindow: 0, maxOutputTokens: 0 } : null;
  },
});
const contextWindow = computed({
  get: () => String(model.value?.contextWindow ?? ""),
  set: (value: string) => {
    if (model.value) model.value.contextWindow = Number(value);
  },
});
const outputLimit = computed({
  get: () => String(model.value?.maxOutputTokens ?? ""),
  set: (value: string) => {
    if (model.value) model.value.maxOutputTokens = Number(value);
  },
});
</script>

<template>
  <section class="mt-5 grid gap-3 border-t border-stroke-divider pt-4">
    <div class="flex items-center justify-between gap-3">
      <label :for="switchId" class="text-label text-fg-primary">{{
        t("settings.ai.models.builtinAgent.enabled")
      }}</label>
      <UiSwitch
        :id="switchId"
        v-model="enabled"
        :disabled="disabled"
        :label="t('settings.ai.models.builtinAgent.enabled')"
      />
    </div>
    <p class="text-caption text-fg-secondary">{{ t("settings.ai.models.builtinAgent.help") }}</p>
    <div v-if="model" class="grid gap-3">
      <label class="grid gap-1 text-caption text-fg-secondary">
        {{ t("settings.ai.models.builtinAgent.contextWindow") }}
        <UiInput v-model="contextWindow" type="number" :disabled="disabled" />
      </label>
      <label class="grid gap-1 text-caption text-fg-secondary">
        {{ t("settings.ai.models.builtinAgent.maxOutputTokens") }}
        <UiInput v-model="outputLimit" type="number" :disabled="disabled" />
      </label>
      <p
        v-if="showErrors && !builtinAgentLimitsValid(model)"
        class="text-caption text-fg-danger"
        role="alert"
      >
        {{ t("settings.ai.models.builtinAgent.invalidLimits") }}
      </p>
    </div>
  </section>
</template>
