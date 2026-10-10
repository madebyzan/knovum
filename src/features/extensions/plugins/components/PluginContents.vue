<script setup lang="ts">
import { Package, Server, Box } from "@lucide/vue";
import { useQuery } from "@tanstack/vue-query";
import { computed, shallowRef, watch } from "vue";

import { getErrorMessage } from "../../../../foundation/errors/error-format";
import { connectorQuery, skillQuery } from "../../../../foundation/extensions/extension-query";
import { copySkill } from "../../../../foundation/extensions/skill-operations";
import type { PluginInstallation } from "../../../../shared/contracts/generated/PluginInstallation";
import type { SkillRevision } from "../../../../shared/contracts/generated/SkillRevision";
import UiSpinner from "../../../../shared/ui/UiSpinner.vue";
import ExtensionCleanupNotice from "../../components/ExtensionCleanupNotice.vue";
import ExtensionIdentity from "../../components/ExtensionIdentity.vue";
import ExtensionInspectionHeader from "../../components/ExtensionInspectionHeader.vue";
import { isAppConnector } from "../../connectors/app-connector";
import type { ExtensionDestination } from "../../extension-destination";
import SkillContents from "../../skills/components/SkillContents.vue";
import { exportSkillDirectory } from "../../skills/skill-directory";
import { useExtensionChanges } from "../../use-extension-changes";
import { usePluginLabels } from "../use-plugin-labels";
import PluginActions from "./PluginActions.vue";
import PluginProvenance from "./PluginProvenance.vue";
const props = defineProps<{ plugin: PluginInstallation; busy: boolean }>();
defineEmits<{ navigate: [destination: ExtensionDestination]; toggle: []; update: []; uninstall: [] }>();
const { describePlugin } = usePluginLabels();
const label = computed(() =>
  describePlugin(
    props.plugin.source,
    props.plugin.revision.manifest.name,
    props.plugin.revision.manifest.description,
  ),
);
const skills = useQuery(skillQuery);
const changes = useExtensionChanges();
function copy(skill: SkillRevision) {
  void changes.mutateAsync(() => copySkill(skill.id)).catch(() => {});
}
function exportSkill(skill: SkillRevision) {
  void changes.mutateAsync(() => exportSkillDirectory(skill.id)).catch(() => {});
}
const connectors = useQuery(connectorQuery);
const ownedSkills = computed(() =>
  (skills.data.value ?? []).filter((skill) => skill.pluginId === props.plugin.id),
);
const selectedSkillId = shallowRef<string>();
const inspectedSkill = computed(() => ownedSkills.value.find((skill) => skill.id === selectedSkillId.value));
watch(
  () => props.plugin.id,
  () => {
    selectedSkillId.value = undefined;
  },
);
const ownedConnectors = computed(() =>
  (connectors.data.value ?? []).filter((connector) => connector.pluginId === props.plugin.id),
);
</script>
<template>
  <section class="space-y-6">
    <ExtensionCleanupNotice :errors="changes.data.value?.cleanupErrors ?? []" />
    <SkillContents
      v-if="inspectedSkill"
      :skill="inspectedSkill"
      :busy="busy || changes.isPending.value"
      @back="selectedSkillId = undefined"
      @copy="copy(inspectedSkill)"
      @export="exportSkill(inspectedSkill)"
    />
    <template v-else>
      <ExtensionInspectionHeader
        :name="label.name"
        :description="label.description"
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
        v-if="skills.error.value || connectors.error.value || changes.error.value"
        role="alert"
        class="text-caption text-fg-danger"
      >
        {{ getErrorMessage(skills.error.value || connectors.error.value || changes.error.value) }}
      </p>
      <div
        v-if="skills.isPending.value || connectors.isPending.value"
        role="status"
        class="flex items-center gap-2 text-caption text-fg-secondary"
      >
        <UiSpinner class="size-4" />{{ $t("common.loading") }}
      </div>
      <section v-if="ownedConnectors.length" class="space-y-2">
        <h3 class="border-b border-stroke-divider pb-2 text-body font-medium">
          {{ $t("extensions.connections.services") }}
          <span class="text-fg-tertiary">{{ ownedConnectors.length }}</span>
        </h3>
        <ExtensionIdentity
          v-for="connector in ownedConnectors"
          :key="connector.id"
          :name="connector.name"
          :description="connector.description"
          :icon="Server"
          @inspect="
            $emit('navigate', {
              tab: isAppConnector(connector, plugin) ? 'apps' : 'mcp',
              connectorId: connector.id,
            })
          "
        />
      </section>
      <section v-if="ownedSkills.length" class="space-y-2">
        <h3 class="border-b border-stroke-divider pb-2 text-body font-medium">
          {{ $t("extensions.overview.skills") }}
          <span class="text-fg-tertiary">{{ ownedSkills.length }}</span>
        </h3>
        <ExtensionIdentity
          v-for="skill in ownedSkills"
          :key="skill.id"
          :name="skill.name"
          :description="skill.description"
          :icon="Box"
          :data-testid="`plugin-skill-${skill.id}`"
          :muted="!plugin.enabled"
          @inspect="selectedSkillId = skill.id"
        />
      </section>
      <PluginProvenance :plugin="plugin" />
      <p class="text-caption text-fg-secondary">{{ $t("extensions.overview.usage") }}</p>
    </template>
    <p v-if="inspectedSkill && changes.error.value" role="alert" class="text-caption text-fg-danger">
      {{ getErrorMessage(changes.error.value) }}
    </p>
  </section>
</template>
