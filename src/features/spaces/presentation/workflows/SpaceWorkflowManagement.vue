<script setup lang="ts">
import { ask } from "@tauri-apps/plugin-dialog";
import { ref, watch } from "vue";
import { useI18n } from "vue-i18n";

import UiButton from "../../../../shared/ui/UiButton.vue";
import UiInput from "../../../../shared/ui/UiInput.vue";
import {
  deleteCapturedWorkflow,
  renameCapturedWorkflow,
  type SavedCapturedWorkflow,
} from "../../workflows/captured-workflow-ipc";
import SpaceWorkflowEditForm from "./SpaceWorkflowEditForm.vue";

const props = defineProps<{ workflow: SavedCapturedWorkflow }>();
const emit = defineEmits<{
  renamed: [value: SavedCapturedWorkflow];
  revised: [value: SavedCapturedWorkflow];
  deleted: [workflowId: string];
}>();
const { t } = useI18n();
const editing = ref(false);
const editingDefinition = ref(false);
const name = ref(props.workflow.name);
const busy = ref(false);
const error = ref("");

watch(
  () => [props.workflow.workflowId, props.workflow.name],
  () => {
    editing.value = false;
    editingDefinition.value = false;
    name.value = props.workflow.name;
    error.value = "";
  },
);

async function rename() {
  if (busy.value || !name.value.trim()) return;
  busy.value = true;
  error.value = "";
  try {
    emit("renamed", await renameCapturedWorkflow(props.workflow.workflowId, props.workflow.name, name.value));
    editing.value = false;
  } catch (cause) {
    error.value = String(cause);
  } finally {
    busy.value = false;
  }
}

async function remove() {
  if (busy.value) return;
  const confirmed = await ask(t("spaces.creation.workflow.deleteConfirm", { name: props.workflow.name }), {
    kind: "warning",
    title: t("spaces.creation.workflow.delete"),
  });
  if (!confirmed) return;
  busy.value = true;
  error.value = "";
  try {
    await deleteCapturedWorkflow(props.workflow.workflowId, props.workflow.name);
    emit("deleted", props.workflow.workflowId);
  } catch (cause) {
    error.value = String(cause);
  } finally {
    busy.value = false;
  }
}
</script>

<template>
  <div class="space-y-2 border-t border-divider pt-3">
    <div v-if="!editing && !editingDefinition" class="flex gap-2">
      <UiButton variant="ghost" size="sm" @click="editingDefinition = true">{{
        t("spaces.creation.workflow.edit")
      }}</UiButton>
      <UiButton variant="ghost" size="sm" @click="editing = true">{{ t("common.rename") }}</UiButton>
      <UiButton variant="dangerSubtle" size="sm" :disabled="busy" @click="remove">{{
        t("spaces.creation.workflow.delete")
      }}</UiButton>
    </div>
    <form v-else-if="editing" class="flex gap-2" @submit.prevent="rename">
      <UiInput v-model="name" :aria-label="t('spaces.creation.workflow.name')" class="min-w-0 flex-1" />
      <UiButton type="submit" variant="primary" size="sm" :disabled="busy || !name.trim()">{{
        t("common.save")
      }}</UiButton>
      <UiButton type="button" variant="ghost" size="sm" @click="editing = false">{{
        t("common.cancel")
      }}</UiButton>
    </form>
    <SpaceWorkflowEditForm
      v-else
      :workflow="workflow"
      @saved="
        emit('revised', $event);
        editingDefinition = false;
      "
      @cancel="editingDefinition = false"
    />
    <p v-if="error" role="alert" class="text-caption text-danger">{{ error }}</p>
  </div>
</template>
