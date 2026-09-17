<script setup lang="ts">
import { isString } from "es-toolkit";
import { computed, reactive, watch } from "vue";
import { useI18n } from "vue-i18n";

import type { CreationWorkflow } from "../../../../foundation/ai/ai-models";
import AiExecutionTargetField from "../../../../foundation/ai/AiExecutionTargetField.vue";
import UiButton from "../../../../shared/ui/UiButton.vue";
import UiDialog from "../../../../shared/ui/UiDialog.vue";
import UiInput from "../../../../shared/ui/UiInput.vue";
import UiSelect from "../../../../shared/ui/UiSelect.vue";
import UiTextarea from "../../../../shared/ui/UiTextarea.vue";
import type { KnowledgeVideoDraft } from "../../knowledge-derivations/video/knowledge-video";
import CreationWorkflowField from "./CreationWorkflowField.vue";

const open = defineModel<boolean>("open", { required: true });
const props = defineProps<{ sourceTitle: string }>();
const emit = defineEmits<{ submit: [draft: KnowledgeVideoDraft] }>();
const { t } = useI18n();
const form = reactive<KnowledgeVideoDraft>({
  title: "",
  prompt: "",
  aspectRatio: "16:9",
  durationSeconds: 5,
});
const workflowParameters = computed(() => ({
  prompt: form.prompt,
  aspectRatio: form.aspectRatio,
  durationSeconds: form.durationSeconds,
}));
const workflowTargets = computed<CreationWorkflow["targets"]>(() => {
  const targets: CreationWorkflow["targets"] = {};
  if (form.textTarget) targets.script = { actionId: "write-script", target: form.textTarget };
  if (form.target) targets.video = { actionId: "generate-video", target: form.target };
  return targets;
});
watch(open, (isOpen) => {
  if (isOpen) {
    form.title = `${props.sourceTitle} · Video`;
    form.prompt = "";
  }
});
function applyWorkflow(workflow: CreationWorkflow) {
  form.textTarget = workflow.targets.script!.target;
  form.target = workflow.targets.video!.target;
  if (isString(workflow.parameters.prompt)) form.prompt = workflow.parameters.prompt;
  if (isVideoAspectRatio(workflow.parameters.aspectRatio)) {
    form.aspectRatio = workflow.parameters.aspectRatio;
  }
  if (isVideoDuration(workflow.parameters.durationSeconds)) {
    form.durationSeconds = workflow.parameters.durationSeconds;
  }
}

const isVideoAspectRatio = (value: unknown): value is KnowledgeVideoDraft["aspectRatio"] =>
  value === "16:9" || value === "9:16" || value === "1:1";
const isVideoDuration = (value: unknown): value is KnowledgeVideoDraft["durationSeconds"] =>
  value === 5 || value === 10 || value === 15;
</script>
<template>
  <UiDialog
    v-model:open="open"
    :title="t('spaces.creation.videoTitle')"
    :description="t('spaces.creation.videoDescription')"
  >
    <div class="space-y-4">
      <UiInput v-model="form.title" :placeholder="t('spaces.creation.videoName')" /><UiTextarea
        v-model="form.prompt"
        :rows="5"
        :placeholder="t('spaces.creation.videoPrompt')"
      />
      <div class="grid grid-cols-2 gap-3">
        <UiSelect
          v-model="form.aspectRatio"
          :label="t('spaces.creation.aspectRatio')"
          :options="[
            { value: '16:9', label: '16:9' },
            { value: '9:16', label: '9:16' },
            { value: '1:1', label: '1:1' },
          ]"
        /><UiSelect
          :model-value="String(form.durationSeconds)"
          :label="t('spaces.creation.videoDuration')"
          :options="[
            { value: '5', label: '5s' },
            { value: '10', label: '10s' },
            { value: '15', label: '15s' },
          ]"
          @update:model-value="
            form.durationSeconds = Number($event) as KnowledgeVideoDraft['durationSeconds']
          "
        />
      </div>
      <AiExecutionTargetField v-model="form.textTarget" purpose="knowledge" />
      <AiExecutionTargetField v-model="form.target" purpose="video-generation" />
      <CreationWorkflowField
        workflow-id="knowledge-video"
        :workflow-version="1"
        :name="form.title"
        :targets="workflowTargets"
        :parameters="workflowParameters"
        :complete="Boolean(form.textTarget && form.target && form.title.trim() && form.prompt.trim())"
        @apply="applyWorkflow"
      />
    </div>
    <template #footer
      ><UiButton variant="ghost" @click="open = false">{{ t("common.cancel") }}</UiButton
      ><UiButton
        variant="primary"
        :disabled="!form.title.trim() || !form.prompt.trim() || !form.textTarget || !form.target"
        @click="emit('submit', { ...form })"
        >{{ t("spaces.creation.generateVideo") }}</UiButton
      ></template
    >
  </UiDialog>
</template>
