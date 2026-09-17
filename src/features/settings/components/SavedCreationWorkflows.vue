<script setup lang="ts">
import { Copy, Pencil, Trash2 } from "@lucide/vue";
import { ask } from "@tauri-apps/plugin-dialog";
import { shallowRef } from "vue";
import { useI18n } from "vue-i18n";

import type { CreationWorkflow } from "../../../foundation/ai/ai-models";
import UiIconButton from "../../../shared/ui/UiIconButton.vue";
import UiInput from "../../../shared/ui/UiInput.vue";
import CreationWorkflowEditor from "./CreationWorkflowEditor.vue";
import SettingsSection from "./SettingsSection.vue";

const workflows = defineModel<CreationWorkflow[]>({ required: true });
const { t } = useI18n();
const editingIndex = shallowRef<number>();
const editorOpen = shallowRef(false);

async function removeWorkflow(index: number) {
  const confirmed = await ask(t("settings.creation.deleteWorkflowConfirm"), {
    title: t("common.delete"),
    kind: "warning",
    okLabel: t("common.delete"),
    cancelLabel: t("common.cancel"),
  });
  if (confirmed) workflows.value.splice(index, 1);
}

function duplicateWorkflow(index: number) {
  const source = workflows.value[index]!;
  workflows.value.splice(index + 1, 0, {
    ...structuredClone(source),
    id: crypto.randomUUID(),
    name: t("settings.creation.copyName", { name: source.name }),
  });
}

function editWorkflow(index: number) {
  editingIndex.value = index;
  editorOpen.value = true;
}

function saveWorkflow(workflow: CreationWorkflow) {
  if (editingIndex.value !== undefined) workflows.value.splice(editingIndex.value, 1, workflow);
}
</script>

<template>
  <SettingsSection
    id="settings-creation-workflows"
    tabindex="-1"
    :title="t('settings.creation.workflows')"
    :description="t('settings.creation.workflowsDescription')"
  >
    <p v-if="workflows.length === 0" class="px-4 py-3 text-caption text-foreground-secondary">
      {{ t("settings.creation.noWorkflows") }}
    </p>
    <div v-else class="divide-y divide-divider">
      <div
        v-for="(workflow, index) in workflows"
        :key="workflow.id"
        class="flex items-center gap-3 px-4 py-3"
      >
        <div class="min-w-0 flex-1">
          <UiInput v-model="workflow.name" :aria-label="t('settings.creation.workflowName')" />
          <p class="mt-0.5 truncate text-caption text-foreground-secondary">
            {{ workflow.workflowId }} · v{{ workflow.workflowVersion }}
          </p>
        </div>
        <UiIconButton :label="t('common.duplicate')" size="sm" @click="duplicateWorkflow(index)">
          <Copy class="size-(--knovum-icon-size-sm)" />
        </UiIconButton>
        <UiIconButton :label="t('common.edit')" size="sm" @click="editWorkflow(index)">
          <Pencil class="size-(--knovum-icon-size-sm)" />
        </UiIconButton>
        <UiIconButton :label="t('common.delete')" size="sm" @click="removeWorkflow(index)">
          <Trash2 class="size-(--knovum-icon-size-sm)" />
        </UiIconButton>
      </div>
    </div>
    <CreationWorkflowEditor
      v-if="editingIndex !== undefined && workflows[editingIndex]"
      v-model:open="editorOpen"
      :workflow="workflows[editingIndex]!"
      @save="saveWorkflow"
    />
  </SettingsSection>
</template>
