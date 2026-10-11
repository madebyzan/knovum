<script setup lang="ts">
import { BookOpen, Bot, Image, Languages, PenLine, ScanSearch, Video } from "@lucide/vue";
import { computed } from "vue";
import { useI18n } from "vue-i18n";

import { availableExecutionTargets } from "../../../foundation/ai/ai-execution";
import { useExecutionCatalogQuery } from "../../../foundation/ai/use-execution-catalog-query";
import type { AiConnectionSettings } from "../../../shared/contracts/generated/AiConnectionSettings";
import type { AiRoutingDefaults } from "../../../shared/contracts/generated/AiRoutingDefaults";
import type { DesktopAgentSettings } from "../../../shared/contracts/generated/DesktopAgentSettings";
import type { RoutingPurpose } from "../../../shared/contracts/generated/RoutingPurpose";
import type { TaskTargetRef } from "../../../shared/contracts/generated/TaskTargetRef";
import UiButton from "../../../shared/ui/UiButton.vue";
import UiSelect from "../../../shared/ui/UiSelect.vue";
import UiSettingsSection from "../../../shared/ui/UiSettingsSection.vue";
import PreferenceField from "./PreferenceField.vue";

const props = defineProps<{
  defaults: AiRoutingDefaults;
  connections: AiConnectionSettings[];
  desktopAgents: DesktopAgentSettings[];
}>();
const emit = defineEmits<{
  requestModelCreation: [purpose: RoutingPurpose];
  updateRoute: [key: keyof AiRoutingDefaults, target: TaskTargetRef | null];
}>();
const { t } = useI18n();
const UNASSIGNED = "__unassigned__";
const catalog = useExecutionCatalogQuery({ detectInstallations: false });
const ROUTE_DEFINITIONS = [
  ["defaultTarget", undefined, Bot],
  ["writing", "writing", PenLine],
  ["knowledge", "knowledge", BookOpen],
  ["visualUnderstanding", "visual-understanding", ScanSearch],
  ["translation", "translation", Languages],
  ["imageGeneration", "image-generation", Image],
  ["videoGeneration", "video-generation", Video],
] as const;
const routeRows = computed(() =>
  ROUTE_DEFINITIONS.map(([key, purpose, icon]) => ({
    key,
    purpose,
    icon,
    label: t(`settings.routing.routes.${key}.label`),
    description: t(`settings.routing.routes.${key}.description`),
    options: availableExecutionTargets(props, purpose).map(({ value, label, target }) => ({
      value,
      target,
      label:
        target.type === "agent"
          ? (catalog.data.value?.agents.find(({ kind }) => kind === target.runtimeKind)?.displayName ?? label)
          : label,
    })),
  })),
);
const routeGroups = computed(() => [
  {
    id: "settings-ai-default-route",
    title: t("settings.routing.defaults.title"),
    rows: routeRows.value.slice(0, 1),
  },
  {
    id: "settings-ai-routing",
    title: t("settings.routing.capabilities.title"),
    description: t("settings.routing.defaults.description"),
    rows: routeRows.value.slice(1),
  },
]);
function update(key: keyof AiRoutingDefaults, value: string) {
  if (value === UNASSIGNED) {
    emit("updateRoute", key, null);
    return;
  }
  const row = routeRows.value.find((route) => route.key === key);
  const option = row?.options.find((candidate) => candidate.value === value);
  if (!option) throw new Error("AI routing target is unavailable");
  emit("updateRoute", key, { ...option.target });
}
function serializeTarget(target: AiRoutingDefaults[keyof AiRoutingDefaults]) {
  return target ? JSON.stringify(target) : UNASSIGNED;
}
</script>

<template>
  <div class="space-y-6">
    <UiSettingsSection
      v-for="group in routeGroups"
      :key="group.id"
      :id="group.id"
      tabindex="-1"
      :title="group.title"
      :description="group.description"
    >
      <PreferenceField
        v-for="row in group.rows"
        :key="row.key"
        :icon="row.icon"
        :label="row.label"
        :description="row.description"
      >
        <UiSelect
          v-if="row.options.length"
          :model-value="serializeTarget(defaults[row.key])"
          :label="row.label"
          :options="[
            {
              value: UNASSIGNED,
              label: t(
                row.key === 'defaultTarget' ? 'settings.routing.unselected' : 'settings.routing.unassigned',
              ),
            },
            ...row.options,
          ]"
          class="w-52"
          @update:model-value="update(row.key, $event)"
        />
        <UiButton
          v-else
          size="sm"
          variant="ghost"
          @click="emit('requestModelCreation', row.purpose ?? 'knowledge')"
        >
          {{ t(connections.length ? "settings.routing.noCompatibleModel" : "settings.ai.connections.add") }}
        </UiButton>
      </PreferenceField>
    </UiSettingsSection>
  </div>
</template>
