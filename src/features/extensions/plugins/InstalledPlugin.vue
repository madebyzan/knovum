<script setup lang="ts">
import { Package } from "@lucide/vue";

import type { PluginInstallation } from "../../../shared/contracts/generated/PluginInstallation";
import ExtensionIdentity from "../components/ExtensionIdentity.vue";
import PluginActions from "./PluginActions.vue";
defineProps<{ plugin: PluginInstallation; busy: boolean }>();
defineEmits<{ inspect: []; toggle: []; update: []; uninstall: [] }>();
</script>
<template>
  <ExtensionIdentity
    :name="plugin.revision.manifest.name"
    :description="plugin.revision.manifest.description"
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
