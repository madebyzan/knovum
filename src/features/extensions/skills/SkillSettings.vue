<script setup lang="ts">
import { Plus, FolderInput } from "@lucide/vue";
import { computed } from "vue";

import { getErrorMessage } from "../../../foundation/errors/error-format";
import { copySkill, enableSkill, disableSkill } from "../../../foundation/extensions/skill-operations";
import UiButton from "../../../shared/ui/UiButton.vue";
import UiDialog from "../../../shared/ui/UiDialog.vue";
import UiSpinner from "../../../shared/ui/UiSpinner.vue";
import ExtensionCleanupNotice from "../components/ExtensionCleanupNotice.vue";
import ExtensionEmptyState from "../components/ExtensionEmptyState.vue";
import type { ExtensionDestination } from "../extension-destination";
import SkillContentEditor from "./SkillContentEditor.vue";
import SkillContents from "./SkillContents.vue";
import SkillOverview from "./SkillOverview.vue";
import SkillResourceBrowser from "./SkillResourceBrowser.vue";
import { useSkillManagement } from "./use-skill-management";
const props = withDefaults(defineProps<{ search?: string; selectedId?: string | undefined }>(), {
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
  importDirectory,
  exportDirectory,
  save,
  closeEditor,
  remove,
} = useSkillManagement();
const skills = computed(() =>
  (query.data.value ?? []).filter((skill) =>
    `${skill.name} ${skill.description}`.toLowerCase().includes(props.search.trim().toLowerCase()),
  ),
);
const inspected = computed(() => query.data.value?.find((skill) => skill.id === props.selectedId));
</script>

<template>
  <section class="min-w-0 space-y-4">
    <header v-if="!selectedId" class="flex flex-wrap justify-end gap-2">
      <UiButton :disabled="busy" data-testid="import-skill" @click="mutate(importDirectory)"
        ><FolderInput class="size-4" />{{ $t("extensions.skills.import") }}</UiButton
      >
      <UiButton :disabled="busy" data-testid="create-skill" @click="create"
        ><Plus class="size-4" />{{ $t("extensions.skills.create") }}</UiButton
      >
    </header>
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
      @navigate="$emit('navigate', $event)"
      @edit="reading.mutate(inspected.id)"
      @toggle="mutate(() => (inspected!.enabled ? disableSkill(inspected!.id) : enableSkill(inspected!.id)))"
      @copy="mutate(() => copySkill(inspected!.id))"
      @export="mutate(() => exportDirectory(inspected!.id))"
      @delete="remove(inspected.id, inspected.name)"
    />
    <div
      v-else-if="query.isPending.value"
      role="status"
      class="flex items-center justify-center gap-2 py-12 text-body text-fg-secondary"
    >
      <UiSpinner class="size-4" />{{ $t("common.loading") }}
    </div>
    <ExtensionEmptyState
      v-else-if="selectedId && !query.error.value"
      :title="$t('extensions.overview.notFound')"
      :description="$t('extensions.overview.notFoundHint')"
      ><UiButton @click="$emit('navigate', { tab: 'skills' })">{{
        $t("extensions.overview.back")
      }}</UiButton></ExtensionEmptyState
    >
    <ExtensionEmptyState
      v-else-if="!query.error.value && !skills.length"
      :title="$t(search ? 'extensions.plugins.search.empty' : 'extensions.skills.empty.title')"
      :description="$t(search ? 'extensions.plugins.search.hint' : 'extensions.skills.empty.description')"
    />
    <div v-else-if="!selectedId" class="divide-y divide-stroke-divider">
      <SkillOverview
        v-for="skill in skills"
        :key="skill.id"
        :skill="skill"
        :busy="busy"
        @inspect="$emit('navigate', { tab: 'skills', id: skill.id })"
        @edit="reading.mutate(skill.id)"
        @toggle="mutate(() => (skill.enabled ? disableSkill(skill.id) : enableSkill(skill.id)))"
        @copy="mutate(() => copySkill(skill.id))"
        @export="mutate(() => exportDirectory(skill.id))"
        @delete="remove(skill.id, skill.name)"
      />
    </div>
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
        :readonly="Boolean(selected?.skill.pluginId)"
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
