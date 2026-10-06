<script setup lang="ts">
import { useMutation, useQuery } from "@tanstack/vue-query";
import { open } from "@tauri-apps/plugin-dialog";
import { shallowRef } from "vue";

import { getErrorMessage } from "../../../foundation/errors/error-format";
import { skillQuery } from "../../../foundation/extensions/extension-query";
import {
  readSkill,
  createSkill,
  saveSkillRevision,
  importSkill,
  copySkill,
  exportSkill,
  setSkillEnabled,
  deleteSkill,
} from "../../../foundation/extensions/skill-operations";
import type { SkillContent } from "../../../shared/contracts/generated/SkillContent";
import UiButton from "../../../shared/ui/UiButton.vue";
import { useExtensionChanges } from "../use-extension-changes";
import SkillContentEditor from "./SkillContentEditor.vue";
import SkillResourceBrowser from "./SkillResourceBrowser.vue";
const query = useQuery(skillQuery);
const changes = useExtensionChanges();
const reading = useMutation({
  mutationFn: readSkill,
  onSuccess: (content) => {
    selected.value = content;
    name.value = content.skill.name;
    markdown.value = content.markdown;
    editing.value = true;
  },
});
const selected = shallowRef<SkillContent>();
const name = shallowRef("");
const markdown = shallowRef("");
const editing = shallowRef(false);
function create() {
  selected.value = undefined;
  name.value = "";
  markdown.value = "---\nname: \ndescription: \n---\n\n";
  editing.value = true;
}
function mutate(operation: () => Promise<unknown>) {
  void changes.mutateAsync(operation).catch(() => {});
}
async function importDirectory() {
  const path = await open({ directory: true, multiple: false });
  if (typeof path === "string") await importSkill(path);
}
async function exportDirectory(id: string) {
  const path = await open({ directory: true, multiple: false });
  if (typeof path === "string") await exportSkill(id, path);
}
async function save() {
  await changes
    .mutateAsync(async () => {
      if (selected.value) await saveSkillRevision(selected.value.skill.id, markdown.value);
      else await createSkill(name.value, markdown.value);
      editing.value = false;
    })
    .catch(() => {});
}
</script>

<template>
  <section class="space-y-3">
    <div class="flex gap-2">
      <UiButton :disabled="changes.isPending.value" data-testid="create-skill" @click="create">{{
        $t("extensions.createSkill")
      }}</UiButton
      ><UiButton :disabled="changes.isPending.value" @click="mutate(importDirectory)">{{
        $t("extensions.import")
      }}</UiButton>
    </div>
    <p
      v-if="query.error.value || changes.error.value || reading.error.value"
      role="alert"
      class="text-body text-fg-danger"
    >
      {{ getErrorMessage(query.error.value || changes.error.value || reading.error.value) }}
    </p>
    <SkillContentEditor
      v-if="editing"
      v-model:name="name"
      v-model:markdown="markdown"
      :existing="Boolean(selected)"
      :readonly="Boolean(selected?.skill.pluginId)"
      :busy="changes.isPending.value"
      :files="selected?.files ?? []"
      @save="save"
      @close="editing = false"
    />
    <SkillResourceBrowser
      v-if="editing && selected"
      :key="selected.skill.digest"
      :skill-id="selected.skill.id"
      :files="selected.files"
    />
    <p v-if="query.isPending.value" class="text-caption text-fg-secondary">{{ $t("extensions.loading") }}</p>
    <p v-else-if="!query.error.value && !query.data.value?.length" class="text-caption text-fg-secondary">
      {{ $t("extensions.emptySkills") }}
    </p>
    <section
      v-for="skill in query.data.value"
      :key="skill.id"
      class="space-y-2 border-b border-stroke-divider py-3"
      :data-testid="`skill-${skill.id}`"
    >
      <h2 class="text-body font-medium">{{ skill.name }}</h2>
      <p class="text-caption text-fg-secondary">{{ skill.description }}</p>
      <p class="text-caption break-all text-fg-tertiary">
        {{ skill.pluginId ?? $t("extensions.personal") }} · {{ skill.digest }}
      </p>
      <div class="flex flex-wrap gap-2">
        <UiButton size="sm" :disabled="reading.isPending.value" @click="reading.mutate(skill.id)">{{
          $t(skill.pluginId ? "extensions.view" : "extensions.edit")
        }}</UiButton>
        <UiButton
          size="sm"
          :disabled="changes.isPending.value"
          @click="mutate(() => setSkillEnabled(skill.id, !skill.enabled))"
          >{{ $t(skill.enabled ? "extensions.disable" : "extensions.enable") }}</UiButton
        >
        <UiButton size="sm" :disabled="changes.isPending.value" @click="mutate(() => copySkill(skill.id))">{{
          $t("extensions.copy")
        }}</UiButton>
        <UiButton
          size="sm"
          :disabled="changes.isPending.value"
          @click="mutate(() => exportDirectory(skill.id))"
          >{{ $t("extensions.export") }}</UiButton
        >
        <UiButton
          v-if="!skill.pluginId"
          size="sm"
          variant="dangerSubtle"
          :disabled="changes.isPending.value"
          @click="mutate(() => deleteSkill(skill.id))"
          >{{ $t("extensions.delete") }}</UiButton
        >
      </div>
    </section>
  </section>
</template>
