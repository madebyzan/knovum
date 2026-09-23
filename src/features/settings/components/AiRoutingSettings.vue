<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";

import type {
  AgentDescriptor,
  AgentInstallation,
  DesktopAgentSettings,
} from "../../../foundation/ai/agent-models";
import {
  availableExecutionTargets,
  decodeExecutionTarget,
  encodeExecutionTarget,
} from "../../../foundation/ai/ai-execution";
import type {
  AiConnectionSettings,
  AiRoutingDefaults,
  AiRoutingPurpose,
} from "../../../foundation/ai/ai-models";
import UiSelect from "../../../shared/ui/UiSelect.vue";
import SettingsRow from "./SettingsRow.vue";
import SettingsSection from "./SettingsSection.vue";

const props = defineProps<{
  connections: AiConnectionSettings[];
  desktopAgents: DesktopAgentSettings[];
  agentDescriptors: readonly AgentDescriptor[];
  agentInstallations: readonly AgentInstallation[];
}>();
const routes = defineModel<AiRoutingDefaults>({ required: true });
const emit = defineEmits<{ changed: [] }>();
const { t } = useI18n();
const routeDefinitions = [
  ["writing", "writing", "writingRoute"],
  ["knowledge", "knowledge", "knowledgeRoute"],
  ["visualUnderstanding", "visual-understanding", "visualUnderstandingRoute"],
  ["translation", "translation", "translationRoute"],
  ["imageGeneration", "image-generation", "imageGenerationRoute"],
  ["videoGeneration", "video-generation", "videoGenerationRoute"],
] as const satisfies readonly (readonly [keyof AiRoutingDefaults, AiRoutingPurpose, string])[];
const settings = computed(() => ({
  connections: props.connections,
  desktopAgents: props.desktopAgents,
  defaults: routes.value,
}));
const routeRows = computed(() =>
  routeDefinitions.map(([key, purpose, translationKey]) => ({
    key,
    purpose,
    label: t(`settings.ai.${translationKey}`),
    description: t(`settings.ai.${translationKey}Description`),
    options: availableExecutionTargets(settings.value, purpose).map((option) =>
      Object.assign({}, option, {
        description:
          option.disabledReason ?? option.statusDetail ?? t("settings.ai.executionTargetModelNative"),
      }),
    ),
  })),
);
const hasTargets = computed(() => props.connections.some((connection) => connection.models.length > 0));

function update(key: keyof AiRoutingDefaults, value: string) {
  const target = value ? decodeExecutionTarget(value) : null;
  routes.value[key] = target;
  emit("changed");
}
const selected = (target: AiRoutingDefaults[keyof AiRoutingDefaults]) =>
  target ? encodeExecutionTarget(target) : "";
</script>

<template>
  <SettingsSection
    id="settings-ai-routing"
    tabindex="-1"
    :title="t('settings.ai.defaultRoutes')"
    :description="t('settings.ai.defaultRoutesDescription')"
  >
    <div v-if="!hasTargets" class="px-4 py-3">
      <p class="text-body-lg font-medium text-foreground">{{ t("settings.ai.routingUnavailable") }}</p>
      <p class="mt-0.5 max-w-xl text-caption leading-normal text-foreground-secondary">
        {{ t("settings.ai.routingUnavailableDescription") }}
      </p>
    </div>
    <template v-else>
      <SettingsRow v-for="row in routeRows" :key="row.key" :label="row.label" :description="row.description">
        <UiSelect
          :model-value="selected(routes[row.key])"
          :label="row.label"
          :options="row.options"
          :placeholder="t('settings.ai.noCompatibleTarget')"
          :disabled="row.options.length === 0"
          @update:model-value="update(row.key, $event)"
        />
      </SettingsRow>
    </template>
  </SettingsSection>
</template>
