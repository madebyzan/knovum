<script setup lang="ts">
import { Copy, Download, MoreHorizontal, Trash2 } from "@lucide/vue";

import type { SkillRevision } from "../../../shared/contracts/generated/SkillRevision";
import UiBadge from "../../../shared/ui/UiBadge.vue";
import UiButton from "../../../shared/ui/UiButton.vue";
import UiDropdownMenu from "../../../shared/ui/UiDropdownMenu.vue";
import UiDropdownMenuItem from "../../../shared/ui/UiDropdownMenuItem.vue";
import UiIconButton from "../../../shared/ui/UiIconButton.vue";
import UiSwitch from "../../../shared/ui/UiSwitch.vue";

defineProps<{ skill: SkillRevision; busy: boolean }>();
defineEmits<{ edit: []; toggle: []; copy: []; export: []; delete: [] }>();
</script>

<template>
  <section
    class="flex min-h-(--knovum-setting-min-height) items-start gap-3 px-4 py-3"
    :data-testid="`skill-${skill.id}`"
  >
    <div class="min-w-0 flex-1">
      <div class="flex flex-wrap items-center gap-2">
        <h3 class="text-body-lg font-medium break-words text-fg-primary">{{ skill.name }}</h3>
        <UiBadge size="sm">{{
          $t(skill.pluginId ? "extensions.pluginSkill" : "extensions.personal")
        }}</UiBadge>
      </div>
      <p class="mt-0.5 text-caption leading-normal text-fg-secondary">{{ skill.description }}</p>
      <p v-if="skill.unsupportedRequirements.length" class="mt-2 text-caption text-fg-warning">
        {{
          $t("extensions.unsupportedRequirements", { requirements: skill.unsupportedRequirements.join(", ") })
        }}
      </p>
      <UiButton
        size="sm"
        variant="ghost"
        class="mt-1 -ml-2.5"
        :disabled="busy"
        :data-testid="`edit-skill-${skill.id}`"
        @click="$emit('edit')"
        >{{ $t(skill.pluginId ? "extensions.view" : "extensions.edit") }}</UiButton
      >
    </div>
    <UiSwitch
      class="mt-1"
      :model-value="skill.enabled"
      :disabled="busy"
      :label="$t(skill.enabled ? 'extensions.disableNamed' : 'extensions.enableNamed', { name: skill.name })"
      :data-testid="`toggle-skill-${skill.id}`"
      @update:model-value="$emit('toggle')"
    />
    <UiDropdownMenu>
      <template #trigger
        ><UiIconButton :label="$t('extensions.skillActions')" :tooltip="false" :disabled="busy"
          ><MoreHorizontal class="size-4" /></UiIconButton
      ></template>
      <UiDropdownMenuItem :disabled="busy" @select="$emit('copy')"
        ><Copy class="size-3.5" />{{ $t("extensions.copy") }}</UiDropdownMenuItem
      >
      <UiDropdownMenuItem :disabled="busy" @select="$emit('export')"
        ><Download class="size-3.5" />{{ $t("extensions.export") }}</UiDropdownMenuItem
      >
      <UiDropdownMenuItem v-if="!skill.pluginId" destructive :disabled="busy" @select="$emit('delete')"
        ><Trash2 class="size-3.5" />{{ $t("extensions.delete") }}</UiDropdownMenuItem
      >
    </UiDropdownMenu>
  </section>
</template>
