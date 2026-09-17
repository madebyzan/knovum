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
import type { MeetingVideoPackageDraft } from "../../knowledge-derivations/workflows/meeting-video-package";
import CreationWorkflowField from "./CreationWorkflowField.vue";

const open = defineModel<boolean>("open", { required: true });
const props = defineProps<{ sourceTitle: string }>();
const emit = defineEmits<{ submit: [draft: MeetingVideoPackageDraft] }>();
const { t } = useI18n();
const form = reactive<MeetingVideoPackageDraft>({ summaryTitle: "", summaryPurpose: "" });
const valid = computed(() =>
  Boolean(form.summaryTitle.trim() && form.summaryPurpose.trim() && form.textTarget),
);
const workflowTargets = computed<CreationWorkflow["targets"]>(() =>
  form.textTarget ? { summary: { actionId: "summarize-meeting", target: form.textTarget } } : {},
);
const workflowParameters = computed(() => ({
  summaryPurpose: form.summaryPurpose,
}));

watch(open, (isOpen) => {
  if (!isOpen) return;
  form.summaryTitle = `${props.sourceTitle} · ${t("spaces.creation.meetingSummaryTitle")}`;
  form.summaryPurpose = t("spaces.creation.meetingSummaryPurpose");
});

function applyWorkflow(workflow: CreationWorkflow) {
  form.textTarget = workflow.targets.summary!.target;
  if (isString(workflow.parameters.summaryPurpose)) {
    form.summaryPurpose = workflow.parameters.summaryPurpose;
  }
}
</script>

<template>
  <UiDialog
    v-model:open="open"
    :title="t('spaces.creation.meetingVideoPackage')"
    :description="t('spaces.creation.meetingVideoDescription')"
  >
    <div class="space-y-4">
      <UiInput v-model="form.summaryTitle" :placeholder="t('spaces.inspector.detailTitle')" />
      <UiTextarea v-model="form.summaryPurpose" :rows="4" />
      <AiExecutionTargetField v-model="form.textTarget" purpose="knowledge" />
      <CreationWorkflowField
        workflow-id="meeting-video-package"
        :workflow-version="1"
        :name="form.summaryTitle"
        :targets="workflowTargets"
        :parameters="workflowParameters"
        :complete="valid"
        @apply="applyWorkflow"
      />
    </div>
    <template #footer>
      <UiButton variant="ghost" @click="open = false">{{ t("common.cancel") }}</UiButton>
      <UiButton variant="primary" :disabled="!valid" @click="emit('submit', { ...form })">
        {{ t("spaces.creation.createMeetingPackage") }}
      </UiButton>
    </template>
  </UiDialog>
</template>
