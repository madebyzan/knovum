<script setup lang="ts">
import { Package, Blocks, Box, Server } from "@lucide/vue";
import { useQuery } from "@tanstack/vue-query";
import { computed } from "vue";

import { getErrorMessage } from "../../../foundation/errors/error-format";
import { connectorQuery, connectionQuery, skillQuery } from "../../../foundation/extensions/extension-query";
import { enableSkill, disableSkill } from "../../../foundation/extensions/skill-operations";
import type { PluginInstallation } from "../../../shared/contracts/generated/PluginInstallation";
import type { SkillRevision } from "../../../shared/contracts/generated/SkillRevision";
import UiSpinner from "../../../shared/ui/UiSpinner.vue";
import UiSwitch from "../../../shared/ui/UiSwitch.vue";
import ExtensionIdentity from "../components/ExtensionIdentity.vue";
import ExtensionInspectionHeader from "../components/ExtensionInspectionHeader.vue";
import type { ExtensionDestination } from "../extension-destination";
import { useExtensionChanges } from "../use-extension-changes";
import PluginActions from "./PluginActions.vue";
import PluginProvenance from "./PluginProvenance.vue";
const props = defineProps<{ plugin: PluginInstallation; busy: boolean }>();
defineEmits<{ navigate: [destination: ExtensionDestination]; toggle: []; update: []; uninstall: [] }>();
const skills = useQuery(skillQuery);
const changes = useExtensionChanges();
function toggleSkill(skill: SkillRevision) {
  void changes
    .mutateAsync(async () => {
      await (skill.enabled ? disableSkill(skill.id) : enableSkill(skill.id));
    })
    .catch(() => {});
}
const connectors = useQuery(connectorQuery);
const connections = useQuery(connectionQuery);
const ownedSkills = computed(() =>
  (skills.data.value ?? []).filter((skill) => skill.pluginId === props.plugin.id),
);
const ownedConnectors = computed(() =>
  (connectors.data.value ?? []).filter((connector) => connector.pluginId === props.plugin.id),
);
const ownedConnections = computed(() =>
  (connections.data.value ?? []).filter((connection) =>
    ownedConnectors.value.some((connector) => connector.id === connection.connectorId),
  ),
);
</script>
<template>
  <section class="space-y-6">
    <ExtensionInspectionHeader
      :name="plugin.revision.manifest.name"
      :description="plugin.revision.manifest.description"
      :icon="Package"
      @back="$emit('navigate', { tab: 'plugins' })"
    >
      <template #status
        ><p v-if="!plugin.enabled" class="mt-2 text-caption text-fg-warning">
          {{ $t("extensions.overview.pluginPaused") }}
        </p></template
      >
      <PluginActions
        :plugin="plugin"
        :busy="busy"
        @toggle="$emit('toggle')"
        @update="$emit('update')"
        @uninstall="$emit('uninstall')"
      />
    </ExtensionInspectionHeader>
    <p
      v-if="skills.error.value || connectors.error.value || connections.error.value || changes.error.value"
      role="alert"
      class="text-caption text-fg-danger"
    >
      {{
        getErrorMessage(
          skills.error.value || connectors.error.value || connections.error.value || changes.error.value,
        )
      }}
    </p>
    <div
      v-if="skills.isPending.value || connectors.isPending.value || connections.isPending.value"
      role="status"
      class="flex items-center gap-2 text-caption text-fg-secondary"
    >
      <UiSpinner class="size-4" />{{ $t("common.loading") }}
    </div>
    <section v-if="ownedConnectors.length" class="space-y-2">
      <h3 class="border-b border-stroke-divider pb-2 text-body font-medium">
        {{ $t("extensions.overview.applications") }}
        <span class="text-fg-tertiary">{{ ownedConnectors.length }}</span>
      </h3>
      <ExtensionIdentity
        v-for="connector in ownedConnectors"
        :key="connector.id"
        :name="connector.name"
        :description="connector.description"
        :icon="Blocks"
        @inspect="$emit('navigate', { tab: 'applications', id: connector.id })"
      />
    </section>
    <section v-if="ownedSkills.length" class="space-y-2">
      <h3 class="border-b border-stroke-divider pb-2 text-body font-medium">
        {{ $t("extensions.overview.skills") }} <span class="text-fg-tertiary">{{ ownedSkills.length }}</span>
      </h3>
      <ExtensionIdentity
        v-for="skill in ownedSkills"
        :key="skill.id"
        :name="skill.name"
        :description="skill.description"
        :icon="Box"
        :muted="!skill.enabled || !plugin.enabled"
        @inspect="$emit('navigate', { tab: 'skills', id: skill.id })"
        ><UiSwitch
          :model-value="skill.enabled"
          :disabled="
            busy ||
            changes.isPending.value ||
            (!skill.enabled && Boolean(skill.unsupportedRequirements.length))
          "
          :label="
            $t(skill.enabled ? 'extensions.plugins.disable' : 'extensions.plugins.enable', {
              name: skill.name,
            })
          "
          @update:model-value="toggleSkill(skill)"
      /></ExtensionIdentity>
    </section>
    <section v-if="ownedConnections.length" class="space-y-2">
      <h3 class="border-b border-stroke-divider pb-2 text-body font-medium">
        MCP <span class="text-fg-tertiary">{{ ownedConnections.length }}</span>
      </h3>
      <ExtensionIdentity
        v-for="connection in ownedConnections"
        :key="connection.id"
        :name="connection.name"
        :icon="Server"
        :muted="!connection.enabled || !plugin.enabled"
        @inspect="$emit('navigate', { tab: 'mcp', id: connection.id })"
      />
    </section>
    <PluginProvenance :plugin="plugin" />
    <p class="text-caption text-fg-secondary">{{ $t("extensions.overview.usage") }}</p>
  </section>
</template>
