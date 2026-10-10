<script setup lang="ts">
import { Box } from "@lucide/vue";
import { useQuery } from "@tanstack/vue-query";
import { computed } from "vue";

import { getErrorMessage } from "../../../foundation/errors/error-format";
import { skillContentQuery } from "../../../foundation/extensions/extension-query";
import type { SkillRevision } from "../../../shared/contracts/generated/SkillRevision";
import UiMarkdown from "../../../shared/ui/UiMarkdown.vue";
import UiSpinner from "../../../shared/ui/UiSpinner.vue";
import ExtensionAvailability from "../components/ExtensionAvailability.vue";
import ExtensionInspectionHeader from "../components/ExtensionInspectionHeader.vue";
import SkillActions from "./SkillActions.vue";
import SkillResourceBrowser from "./SkillResourceBrowser.vue";
const props = defineProps<{ skill: SkillRevision; busy: boolean }>();
defineEmits<{
  back: [];
  edit: [];
  toggle: [];
  copy: [];
  export: [];
  delete: [];
}>();
const query = useQuery(computed(() => skillContentQuery(props.skill.id, props.skill.digest)));
const instructions = computed(
  () => query.data.value?.markdown.replace(/^---\r?\n[\s\S]*?\r?\n---(?:\r?\n|$)/, "").trim() ?? "",
);
</script>
<template>
  <section class="space-y-6">
    <ExtensionInspectionHeader
      :name="skill.name"
      :description="skill.description"
      :icon="Box"
      @back="$emit('back')"
    >
      <template #status
        ><ExtensionAvailability
          class="mt-2"
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
    </ExtensionInspectionHeader>
    <p v-if="skill.pluginId" class="text-caption text-fg-secondary">{{ $t("extensions.skills.readOnly") }}</p>
    <p v-if="skill.unsupportedRequirements.length" class="text-caption text-fg-warning">
      {{
        $t("extensions.skills.unsupportedRequirements", {
          requirements: skill.unsupportedRequirements.join(", "),
        })
      }}
    </p>
    <p v-if="skill.requirements.length" class="text-caption text-fg-secondary">
      {{ $t("extensions.skills.requiredConnections", { names: skill.requirements.join(", ") }) }}
    </p>
    <p v-if="query.error.value" role="alert" class="text-body text-fg-danger">
      {{ getErrorMessage(query.error.value) }}
    </p>
    <UiSpinner v-else-if="query.isPending.value" class="size-4" />
    <template v-else-if="query.data.value">
      <section class="space-y-4">
        <h3 class="border-b border-stroke-divider pb-2 text-body font-medium">
          {{ $t("extensions.skills.instructions") }}
        </h3>
        <UiMarkdown :content="instructions" />
      </section>
      <SkillResourceBrowser :key="skill.digest" :skill-id="skill.id" :files="query.data.value.files" />
    </template>
    <p class="text-caption text-fg-secondary">{{ $t("extensions.overview.usage") }}</p>
  </section>
</template>
