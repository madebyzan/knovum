<script setup lang="ts">
import { Package } from "@lucide/vue";
import { computed } from "vue";

import type { PluginInstallation } from "../../../../shared/contracts/generated/PluginInstallation";
import ExtensionIdentity from "../../components/ExtensionIdentity.vue";
import { usePluginLabels } from "../use-plugin-labels";
import PluginActions from "./PluginActions.vue";
const props = defineProps<{ plugin: PluginInstallation; busy: boolean }>();
defineEmits<{ inspect: []; toggle: []; update: []; uninstall: [] }>();
const { describePlugin } = usePluginLabels();
const label = computed(() =>
  describePlugin(
    props.plugin.source,
    props.plugin.revision.manifest.name,
    props.plugin.revision.manifest.description,
  ),
);
</script>
<template>
  <ExtensionIdentity
    :name="label.name"
    :description="label.description"
    :icon="Package"
    :muted="!plugin.enabled"
    :data-testid="`plugin-${plugin.id}`"
    @inspect="$emit('inspect')"
  >
    <template #status
      ><span v-if="plugin.builtin" class="mt-1 block text-caption text-fg-tertiary">{{
        $t("extensions.plugins.bundled")
      }}</span></template
    >
    <PluginActions
      :plugin="plugin"
      :busy="busy"
      @toggle="$emit('toggle')"
      @update="$emit('update')"
      @uninstall="$emit('uninstall')"
    />
  </ExtensionIdentity>
</template>
