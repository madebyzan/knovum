<script setup lang="ts">
import { computed, onMounted, ref, shallowRef, watch } from "vue";
import { useI18n } from "vue-i18n";

import type { AiRoutingPurpose, CreationWorkflow } from "../../../foundation/ai/ai-models";
import AiExecutionTargetField from "../../../foundation/ai/AiExecutionTargetField.vue";
import {
  loadKnowledgeActions,
  type KnowledgeActionDefinition,
} from "../../../foundation/ai/knowledge-actions";
import UiButton from "../../../shared/ui/UiButton.vue";
import UiDialog from "../../../shared/ui/UiDialog.vue";
import UiInput from "../../../shared/ui/UiInput.vue";
import CreationParameterFields from "./CreationParameterFields.vue";

const open = defineModel<boolean>("open", { required: true });
const props = defineProps<{ workflow: CreationWorkflow }>();
const emit = defineEmits<{ save: [workflow: CreationWorkflow] }>();
const { t } = useI18n();
const draft = ref<CreationWorkflow>(structuredClone(props.workflow));
const actions = shallowRef<KnowledgeActionDefinition[]>([]);
const actionLoadError = shallowRef<unknown>();

onMounted(async () => {
  try {
    actions.value = await loadKnowledgeActions();
  } catch (error) {
    actionLoadError.value = error;
  }
});

watch(open, (value) => {
  if (value) draft.value = structuredClone(props.workflow);
});

const valid = computed(
  () =>
    actions.value.length > 0 &&
    draft.value.name.trim().length > 0 &&
    Object.values(draft.value.targets).every(({ target }) => target),
);

function save() {
  if (!valid.value) return;
  emit("save", { ...structuredClone(draft.value), name: draft.value.name.trim() });
  open.value = false;
}

function routingPurpose(actionId: string): AiRoutingPurpose {
  const action = actions.value.find((candidate) => candidate.id === actionId);
  if (!action?.routingPurpose) throw new Error(`Knowledge Action has no execution target: ${actionId}`);
  return action.routingPurpose;
}
</script>

<template>
  <UiDialog
    v-model:open="open"
    :title="t('settings.creation.editWorkflow')"
    :description="`${draft.workflowId} · v${draft.workflowVersion}`"
  >
    <div class="grid gap-4">
      <label class="grid gap-1.5 text-label font-medium">
        {{ t("settings.creation.workflowName") }}
        <UiInput v-model="draft.name" />
      </label>
      <p v-if="actionLoadError" class="text-caption text-danger">{{ String(actionLoadError) }}</p>
      <section v-else-if="actions.length" class="grid gap-2">
        <h3 class="text-label font-medium">{{ t("settings.creation.executionTargets") }}</h3>
        <div v-for="(binding, step) in draft.targets" :key="step" class="grid gap-1.5">
          <span class="text-caption text-foreground-secondary">{{ step }}</span>
          <AiExecutionTargetField v-model="binding.target" :purpose="routingPurpose(binding.actionId)" />
        </div>
      </section>
      <section v-if="Object.keys(draft.parameters).length" class="grid gap-2">
        <h3 class="text-label font-medium">{{ t("settings.creation.parameters") }}</h3>
        <CreationParameterFields v-model="draft.parameters" />
      </section>
    </div>
    <template #footer>
      <UiButton variant="ghost" @click="open = false">{{ t("common.cancel") }}</UiButton>
      <UiButton variant="primary" :disabled="!valid" @click="save">{{ t("common.save") }}</UiButton>
    </template>
  </UiDialog>
</template>
