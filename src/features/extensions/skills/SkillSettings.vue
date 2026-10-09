<script setup lang="ts">
import { Plus, FolderInput } from "@lucide/vue";
import { computed, shallowRef } from "vue";

import { getErrorMessage } from "../../../foundation/errors/error-format";
import { copySkill, enableSkill, disableSkill } from "../../../foundation/extensions/skill-operations";
import UiButton from "../../../shared/ui/UiButton.vue";
import UiDialog from "../../../shared/ui/UiDialog.vue";
import UiSearchInput from "../../../shared/ui/UiSearchInput.vue";
import UiSettingsSection from "../../../shared/ui/UiSettingsSection.vue";
import UiSpinner from "../../../shared/ui/UiSpinner.vue";
import ExtensionCleanupNotice from "../components/ExtensionCleanupNotice.vue";
import SkillContentEditor from "./SkillContentEditor.vue";
import SkillOverview from "./SkillOverview.vue";
import SkillResourceBrowser from "./SkillResourceBrowser.vue";
import { useSkillManagement } from "./use-skill-management";
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
const search = shallowRef("");
const skills = computed(() =>
  (query.data.value ?? []).filter((skill) =>
    `${skill.name} ${skill.description}`.toLowerCase().includes(search.value.trim().toLowerCase()),
  ),
);
</script>

<template>
  <section class="min-w-0 space-y-4">
    <header class="flex flex-wrap items-end justify-between gap-3 px-0.5">
      <div class="min-w-0 flex-1 basis-48">
        <h2 class="text-title-sm font-semibold tracking-title text-fg-primary">
          {{ $t("extensions.overview.skills") }}
        </h2>
        <p class="mt-1 text-caption text-fg-secondary">{{ $t("extensions.skills.description") }}</p>
      </div>
      <div class="flex flex-wrap gap-2">
        <UiButton :disabled="busy" data-testid="import-skill" @click="mutate(importDirectory)"
          ><FolderInput class="size-4" />{{ $t("extensions.skills.import") }}</UiButton
        ><UiButton :disabled="busy" data-testid="create-skill" @click="create"
          ><Plus class="size-4" />{{ $t("extensions.skills.create") }}</UiButton
        >
      </div>
    </header>
    <p
      v-if="query.error.value || changes.error.value || reading.error.value"
      role="alert"
      class="text-body text-fg-danger"
    >
      {{ getErrorMessage(query.error.value || changes.error.value || reading.error.value) }}
    </p>
    <ExtensionCleanupNotice :errors="changes.data.value?.cleanupErrors ?? []" />
    <UiSearchInput
      v-if="query.data.value?.length"
      v-model="search"
      :label="$t('extensions.skills.search')"
      :placeholder="$t('extensions.skills.search')"
      data-testid="skill-search"
    />
    <div
      v-if="query.isPending.value"
      role="status"
      class="flex items-center justify-center gap-2 py-12 text-body text-fg-secondary"
    >
      <UiSpinner class="size-4" />{{ $t("common.loading") }}
    </div>
    <UiSettingsSection v-else-if="!query.error.value && !skills.length">
      <div class="px-5 py-6">
        <h3 class="text-body-lg font-medium text-fg-primary">
          {{ $t(search ? "extensions.plugins.search.empty" : "extensions.skills.empty.title") }}
        </h3>
        <p class="mt-1 text-caption text-fg-secondary">
          {{ $t(search ? "extensions.plugins.search.hint" : "extensions.skills.empty.description") }}
        </p>
      </div>
    </UiSettingsSection>
    <UiSettingsSection v-else-if="skills.length">
      <div class="divide-y divide-stroke-divider">
        <SkillOverview
          v-for="skill in skills"
          :key="skill.id"
          :skill="skill"
          :busy="busy"
          @edit="reading.mutate(skill.id)"
          @toggle="mutate(() => (skill.enabled ? disableSkill(skill.id) : enableSkill(skill.id)))"
          @copy="mutate(() => copySkill(skill.id))"
          @export="mutate(() => exportDirectory(skill.id))"
          @delete="remove(skill.id, skill.name)"
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
