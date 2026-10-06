<script setup lang="ts">
import { BookOpen, Bot, Image, Languages, PenLine, ScanSearch, Video } from "@lucide/vue";
import { computed } from "vue";
import { useI18n } from "vue-i18n";

import { availableExecutionTargets } from "../../../foundation/ai/ai-execution";
import { useExecutionCatalogQuery } from "../../../foundation/ai/execution-catalog-query";
import { encodeExecutionTarget } from "../../../foundation/ai/execution-target-codec";
import type { AiConnectionSettings } from "../../../shared/contracts/generated/AiConnectionSettings";
import type { AiRoutingDefaults } from "../../../shared/contracts/generated/AiRoutingDefaults";
import type { DesktopAgentSettings } from "../../../shared/contracts/generated/DesktopAgentSettings";
import type { RoutingPurpose } from "../../../shared/contracts/generated/RoutingPurpose";
import type { TaskTargetRef } from "../../../shared/contracts/generated/TaskTargetRef";
import UiSelect from "../../../shared/ui/UiSelect.vue";
import SettingsSection from "./SettingsSection.vue";

const props = defineProps<{
  defaults: AiRoutingDefaults;
  connections: AiConnectionSettings[];
  desktopAgents: DesktopAgentSettings[];
}>();
const emit = defineEmits<{
  manageModels: [purpose: RoutingPurpose];
  updateRoute: [key: keyof AiRoutingDefaults, target: TaskTargetRef | null];
}>();
const { t } = useI18n();
const UNASSIGNED = "__unassigned__";
const catalog = useExecutionCatalogQuery({ detectInstallations: false });
const routeDefinitions = [
  ["defaultTarget", undefined, Bot],
  ["writing", "writing", PenLine],
  ["knowledge", "knowledge", BookOpen],
  ["visualUnderstanding", "visual-understanding", ScanSearch],
  ["translation", "translation", Languages],
  ["imageGeneration", "image-generation", Image],
  ["videoGeneration", "video-generation", Video],
] as const;
const routeRows = computed(() =>
  routeDefinitions.map(([key, purpose, icon]) => ({
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
const selected = (target: AiRoutingDefaults[keyof AiRoutingDefaults]) =>
  target ? encodeExecutionTarget(target) : UNASSIGNED;
</script>

<template>
  <SettingsSection
    id="settings-ai-routing"
    tabindex="-1"
    :title="t('settings.routing.defaultsTitle')"
    :description="t('settings.routing.defaultsDescription')"
  >
    <div
      v-for="row in routeRows"
      :key="row.key"
      class="flex min-h-(--knovum-setting-min-height) flex-wrap items-center justify-between gap-x-6 gap-y-2 border-b border-stroke-divider px-4 py-3 last:border-b-0"
    >
      <div class="flex min-w-0 flex-1 items-center gap-3">
        <component :is="row.icon" class="size-4 shrink-0 text-fg-secondary" aria-hidden="true" />
        <div class="min-w-0">
          <p class="text-label font-medium text-fg-primary">{{ row.label }}</p>
          <p class="mt-0.5 text-caption leading-normal text-fg-secondary">{{ row.description }}</p>
        </div>
      </div>
      <UiSelect
        v-if="row.options.length"
        :model-value="selected(defaults[row.key])"
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
        class="w-full sm:w-64"
        @update:model-value="update(row.key, $event)"
      />
      <button
        v-else
        type="button"
        class="rounded-md text-caption text-fg-secondary underline-offset-2 hover:text-fg-primary hover:underline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-stroke-focus"
        @click="emit('manageModels', row.purpose ?? 'knowledge')"
      >
        {{ t(connections.length ? "settings.routing.noCompatibleModel" : "settings.ai.addConnection") }}
      </button>
    </div>
  </SettingsSection>
</template>
