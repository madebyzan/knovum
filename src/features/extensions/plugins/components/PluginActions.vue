<script setup lang="ts">
import { MoreHorizontal, RefreshCw, Trash2 } from "@lucide/vue";

import type { PluginInstallation } from "../../../../shared/contracts/generated/PluginInstallation";
import UiDropdownMenu from "../../../../shared/ui/UiDropdownMenu.vue";
import UiDropdownMenuItem from "../../../../shared/ui/UiDropdownMenuItem.vue";
import UiIconButton from "../../../../shared/ui/UiIconButton.vue";
import UiSwitch from "../../../../shared/ui/UiSwitch.vue";
defineProps<{ plugin: PluginInstallation; busy: boolean }>();
defineEmits<{ toggle: []; update: []; uninstall: [] }>();
</script>
<template>
  <UiDropdownMenu>
    <template #trigger
      ><UiIconButton
        :label="$t('extensions.plugins.actions')"
        :tooltip="false"
        :disabled="busy"
        :data-testid="`plugin-actions-${plugin.id}`"
        ><MoreHorizontal class="size-4" /></UiIconButton
    ></template>
    <UiDropdownMenuItem
      v-if="!plugin.builtin"
      :disabled="busy"
      :data-testid="`update-plugin-${plugin.id}`"
      @select="$emit('update')"
      ><RefreshCw class="size-3.5" />{{ $t("extensions.plugins.update.check") }}</UiDropdownMenuItem
    >
    <UiDropdownMenuItem
      destructive
      :disabled="busy"
      :data-testid="`uninstall-plugin-${plugin.id}`"
      @select="$emit('uninstall')"
      ><Trash2 class="size-3.5" />{{ $t("extensions.plugins.uninstall.action") }}</UiDropdownMenuItem
    >
  </UiDropdownMenu>
  <UiSwitch
    :model-value="plugin.enabled"
    :disabled="busy"
    :data-testid="`toggle-plugin-${plugin.id}`"
    :label="
      $t(plugin.enabled ? 'extensions.plugins.disable' : 'extensions.plugins.enable', {
        name: plugin.revision.manifest.name,
      })
    "
    @update:model-value="$emit('toggle')"
  />
</template>
