<script setup lang="ts">
import { isString } from "es-toolkit";
import { computed, reactive, watch } from "vue";
import { useI18n } from "vue-i18n";

import type { CreationWorkflow } from "../../../../foundation/ai/ai-models";
import AiExecutionTargetField from "../../../../foundation/ai/AiExecutionTargetField.vue";
import UiButton from "../../../../shared/ui/UiButton.vue";
import UiDialog from "../../../../shared/ui/UiDialog.vue";
import UiInput from "../../../../shared/ui/UiInput.vue";
import UiTextarea from "../../../../shared/ui/UiTextarea.vue";
import type { KnowledgePresentationDraft } from "../../knowledge-derivations/workflows/knowledge-presentation";
import CreationWorkflowField from "./CreationWorkflowField.vue";

const open = defineModel<boolean>("open", { required: true });
const props = defineProps<{ sourceTitle: string }>();
const emit = defineEmits<{ submit: [draft: KnowledgePresentationDraft] }>();
const { t } = useI18n();
const form = reactive<KnowledgePresentationDraft>({ title: "", purpose: "" });
const valid = computed(() => Boolean(form.title.trim() && form.purpose.trim() && form.textTarget));
const targets = computed<CreationWorkflow["targets"]>(() =>
  form.textTarget ? { document: { actionId: "structure-document", target: form.textTarget } } : {},
);
const parameters = computed(() => ({ purpose: form.purpose }));

watch(open, (isOpen) => {
  if (!isOpen) return;
  form.title = `${props.sourceTitle} · ${t("spaces.creation.presentationOutlineTitle")}`;
  form.purpose = t("spaces.creation.presentationOutlinePurpose");
});

function applyWorkflow(workflow: CreationWorkflow) {
  form.textTarget = workflow.targets.document!.target;
  if (isString(workflow.parameters.purpose)) form.purpose = workflow.parameters.purpose;
}
</script>

<template>
  <UiDialog
    v-model:open="open"
    :title="t('spaces.creation.createPresentation')"
    :description="t('spaces.creation.presentationDescription')"
  >
    <div class="space-y-4">
      <UiInput v-model="form.title" :placeholder="t('spaces.inspector.detailTitle')" />
      <UiTextarea v-model="form.purpose" :rows="4" />
      <AiExecutionTargetField v-model="form.textTarget" purpose="knowledge" />
      <CreationWorkflowField
        workflow-id="knowledge-presentation"
        :workflow-version="1"
        :name="form.title"
        :targets="targets"
        :parameters="parameters"
        :complete="valid"
        @apply="applyWorkflow"
      />
    </div>
    <template #footer>
      <UiButton variant="ghost" @click="open = false">{{ t("common.cancel") }}</UiButton>
      <UiButton variant="primary" :disabled="!valid" @click="emit('submit', { ...form })">
        {{ t("spaces.creation.createPresentation") }}
      </UiButton>
    </template>
  </UiDialog>
</template>
