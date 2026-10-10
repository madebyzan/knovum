<script setup lang="ts">
import { Box } from "@lucide/vue";

import type { SkillRevision } from "../../../shared/contracts/generated/SkillRevision";
import ExtensionAvailability from "../components/ExtensionAvailability.vue";
import ExtensionIdentity from "../components/ExtensionIdentity.vue";
import SkillActions from "./SkillActions.vue";
defineProps<{ skill: SkillRevision; busy: boolean }>();
defineEmits<{ inspect: []; edit: []; toggle: []; copy: []; export: []; delete: [] }>();
</script>
<template>
  <ExtensionIdentity
    :name="skill.name"
    :description="skill.description"
    :icon="Box"
    :muted="!skill.enabled"
    :data-testid="`skill-${skill.id}`"
    @inspect="$emit('inspect')"
  >
    <template #status
      ><ExtensionAvailability
        class="mt-1"
        kind="skills"
        :extension-id="skill.id"
        :enabled="skill.enabled"
        :plugin-id="skill.pluginId"
    /></template>
    <SkillActions
      :skill="skill"
      :busy="busy"
      @edit="$emit('edit')"
      @toggle="$emit('toggle')"
      @copy="$emit('copy')"
      @export="$emit('export')"
      @delete="$emit('delete')"
    />
  </ExtensionIdentity>
</template>
