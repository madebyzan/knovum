<script setup lang="ts">
import { Plus, FolderInput } from "@lucide/vue";
import { computed } from "vue";

import { getErrorMessage } from "../../../../foundation/errors/error-format";
import { copySkill, enableSkill, disableSkill } from "../../../../foundation/extensions/skill-operations";
import UiButton from "../../../../shared/ui/UiButton.vue";
import UiDialog from "../../../../shared/ui/UiDialog.vue";
import UiSettingsHeaderActions from "../../../../shared/ui/UiSettingsHeaderActions.vue";
import UiSettingsSection from "../../../../shared/ui/UiSettingsSection.vue";
import UiSpinner from "../../../../shared/ui/UiSpinner.vue";
import ExtensionCleanupNotice from "../../components/ExtensionCleanupNotice.vue";
import ExtensionEmptyState from "../../components/ExtensionEmptyState.vue";
import type { ExtensionDestination } from "../../extension-destination";
import { importSkillDirectory, exportSkillDirectory } from "../skill-directory";
import { useSkillManagement } from "../use-skill-management";
import SkillContentEditor from "./SkillContentEditor.vue";
import SkillContents from "./SkillContents.vue";
import SkillOverview from "./SkillOverview.vue";
import SkillResourceBrowser from "./SkillResourceBrowser.vue";
const props = withDefaults(defineProps<{ search?: string; skillId?: string | undefined }>(), {
  search: "",
});
defineEmits<{ navigate: [destination: ExtensionDestination] }>();
const {
  query,
  changes,
  reading,
  selected,
  name,
  markdown,
  editing,
  busy,
  create,
  mutate,
  save,
  closeEditor,
  deletePersonalSkill,
} = useSkillManagement();
const skills = computed(() =>
  (query.data.value ?? []).filter(
    (skill) =>
      !skill.pluginId &&
      `${skill.name} ${skill.description}`.toLowerCase().includes(props.search.trim().toLowerCase()),
  ),
);
const inspected = computed(() =>
  query.data.value?.find((skill) => !skill.pluginId && skill.id === props.skillId),
);
</script>

<template>
  <section class="min-w-0 space-y-4">
    <UiSettingsHeaderActions v-if="!skillId">
      <UiButton :disabled="busy" size="sm" data-testid="import-skill" @click="mutate(importSkillDirectory)"
        ><FolderInput class="size-4" />{{ $t("extensions.skills.import") }}</UiButton
      >
      <UiButton :disabled="busy" size="sm" data-testid="create-skill" @click="create"
        ><Plus class="size-4" />{{ $t("extensions.skills.create") }}</UiButton
      >
    </UiSettingsHeaderActions>
    <p
      v-if="query.error.value || changes.error.value || reading.error.value"
      role="alert"
      class="text-body text-fg-danger"
    >
      {{ getErrorMessage(query.error.value || changes.error.value || reading.error.value) }}
    </p>
    <ExtensionCleanupNotice :errors="changes.data.value?.cleanupErrors ?? []" />
    <SkillContents
      v-if="inspected"
      :skill="inspected"
      :busy="busy"
      @back="$emit('navigate', { tab: 'skills' })"
      @edit="reading.mutate(inspected.id)"
      @toggle="mutate(() => (inspected!.enabled ? disableSkill(inspected!.id) : enableSkill(inspected!.id)))"
      @copy="mutate(() => copySkill(inspected!.id))"
      @export="mutate(() => exportSkillDirectory(inspected!.id))"
      @delete="deletePersonalSkill(inspected.id, inspected.name)"
    />
    <div
      v-else-if="query.isPending.value"
      role="status"
      class="flex items-center justify-center gap-2 py-12 text-body text-fg-secondary"
    >
      <UiSpinner class="size-4" />{{ $t("common.loading") }}
    </div>
    <ExtensionEmptyState
      v-else-if="skillId && !query.error.value"
      :title="$t('extensions.overview.notFound.title')"
      :description="$t('extensions.overview.notFound.description')"
      ><UiButton @click="$emit('navigate', { tab: 'skills' })">{{
        $t("extensions.overview.back")
      }}</UiButton></ExtensionEmptyState
    >
    <ExtensionEmptyState
      v-else-if="!query.error.value && !skills.length"
      :title="$t(search ? 'common.search.empty' : 'extensions.skills.empty.title')"
      :description="$t(search ? 'common.search.hint' : 'extensions.skills.empty.description')"
    />
    <UiSettingsSection v-else-if="!skillId">
      <div class="divide-y divide-stroke-divider px-4">
        <SkillOverview
          v-for="skill in skills"
          :key="skill.id"
          :skill="skill"
          :busy="busy"
          @inspect="$emit('navigate', { tab: 'skills', id: skill.id })"
          @edit="reading.mutate(skill.id)"
          @toggle="mutate(() => (skill.enabled ? disableSkill(skill.id) : enableSkill(skill.id)))"
          @copy="mutate(() => copySkill(skill.id))"
          @export="mutate(() => exportSkillDirectory(skill.id))"
          @delete="deletePersonalSkill(skill.id, skill.name)"
        />
      </div>
    </UiSettingsSection>
    <UiDialog
      :open="editing"
      :title="selected ? selected.skill.name : $t('extensions.skills.create')"
      size="wide"
      :closable="!changes.isPending.value"
      @update:open="!$event && closeEditor()"
    >
      <SkillContentEditor
        v-model:name="name"
        v-model:markdown="markdown"
        :existing="Boolean(selected)"
        :busy="changes.isPending.value"
        :files="selected?.files ?? []"
        @save="save"
        @close="closeEditor"
      />
      <SkillResourceBrowser
        v-if="selected"
        :key="selected.skill.digest"
        :skill-id="selected.skill.id"
        :files="selected.files"
      />
      <p v-if="changes.error.value" role="alert" class="mt-3 text-body text-fg-danger">
        {{ getErrorMessage(changes.error.value) }}
      </p>
    </UiDialog>
  </section>
</template>
