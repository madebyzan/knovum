<script setup lang="ts">
import { Copy, Download, MoreHorizontal, Pencil, Trash2 } from "@lucide/vue";

import type { SkillRevision } from "../../../shared/contracts/generated/SkillRevision";
import UiDropdownMenu from "../../../shared/ui/UiDropdownMenu.vue";
import UiDropdownMenuItem from "../../../shared/ui/UiDropdownMenuItem.vue";
import UiIconButton from "../../../shared/ui/UiIconButton.vue";
import UiSwitch from "../../../shared/ui/UiSwitch.vue";
defineProps<{ skill: SkillRevision; busy: boolean }>();
defineEmits<{ edit: []; toggle: []; copy: []; export: []; delete: [] }>();
</script>
<template>
  <UiDropdownMenu>
    <template #trigger
      ><UiIconButton :label="$t('extensions.skills.actions')" :tooltip="false" :disabled="busy"
        ><MoreHorizontal class="size-4" /></UiIconButton
    ></template>
    <UiDropdownMenuItem
      v-if="!skill.pluginId"
      :disabled="busy"
      :data-testid="`edit-skill-${skill.id}`"
      @select="$emit('edit')"
      ><Pencil class="size-3.5" />{{ $t("common.edit") }}</UiDropdownMenuItem
    >
    <UiDropdownMenuItem :disabled="busy" @select="$emit('copy')"
      ><Copy class="size-3.5" />{{ $t("extensions.skills.copy") }}</UiDropdownMenuItem
    >
    <UiDropdownMenuItem :disabled="busy" @select="$emit('export')"
      ><Download class="size-3.5" />{{ $t("common.export") }}</UiDropdownMenuItem
    >
    <UiDropdownMenuItem v-if="!skill.pluginId" destructive :disabled="busy" @select="$emit('delete')"
      ><Trash2 class="size-3.5" />{{ $t("common.delete") }}</UiDropdownMenuItem
    >
  </UiDropdownMenu>
  <UiSwitch
    v-if="!skill.pluginId"
    :model-value="skill.enabled"
    :disabled="busy || (!skill.enabled && Boolean(skill.unsupportedRequirements.length))"
    :label="
      $t(skill.enabled ? 'extensions.plugins.disable' : 'extensions.plugins.enable', { name: skill.name })
    "
    :data-testid="`toggle-skill-${skill.id}`"
    @update:model-value="$emit('toggle')"
  />
</template>
