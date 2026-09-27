<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";

import type {
  AgentDescriptor,
  AgentInstallation,
  DesktopAgentSettings,
} from "../../../foundation/ai/agent-models";
import { availableExecutionTargets } from "../../../foundation/ai/ai-execution";
import type {
  AiConnectionSettings,
  AiRoutingDefaults,
  AiRoutingPurpose,
} from "../../../foundation/ai/ai-models";
import { decodeExecutionTarget, encodeExecutionTarget } from "../../../foundation/ai/execution-target-codec";
import UiSelect from "../../../shared/ui/UiSelect.vue";
import PreferenceField from "./PreferenceField.vue";
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
  ["writing", "writing"],
  ["knowledge", "knowledge"],
  ["visualUnderstanding", "visual-understanding"],
  ["translation", "translation"],
  ["imageGeneration", "image-generation"],
  ["videoGeneration", "video-generation"],
] as const satisfies readonly (readonly [keyof AiRoutingDefaults, AiRoutingPurpose])[];
const settings = computed(() => ({
  connections: props.connections,
  desktopAgents: props.desktopAgents,
  defaults: routes.value,
}));
const routeRows = computed(() =>
  routeDefinitions.map(([key, purpose]) => ({
    key,
    purpose,
    label: t(`settings.routing.routes.${key}.label`),
    description: t(`settings.routing.routes.${key}.description`),
    options: availableExecutionTargets(settings.value, purpose).map((option) =>
      Object.assign({}, option, {
        description: t("ai.targetModelNative"),
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
    :title="t('settings.routing.defaultsTitle')"
    :description="t('settings.routing.defaultsDescription')"
  >
    <div v-if="!hasTargets" class="px-4 py-3">
      <p class="text-body-lg font-medium text-foreground">{{ t("settings.routing.unavailable") }}</p>
      <p class="mt-0.5 max-w-xl text-caption leading-normal text-foreground-secondary">
        {{ t("settings.routing.unavailableDescription") }}
      </p>
    </div>
    <template v-else>
      <PreferenceField
        v-for="row in routeRows"
        :key="row.key"
        :label="row.label"
        :description="row.description"
      >
        <UiSelect
          :model-value="selected(routes[row.key])"
          :label="row.label"
          :options="row.options"
          :placeholder="t('ai.noCompatibleTarget')"
          :disabled="row.options.length === 0"
          @update:model-value="update(row.key, $event)"
        />
      </PreferenceField>
    </template>
  </SettingsSection>
</template>
