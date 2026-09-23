<script setup lang="ts">
import { computed, shallowRef } from "vue";
import { useI18n } from "vue-i18n";

import UiButton from "../../../../shared/ui/UiButton.vue";
import UiInput from "../../../../shared/ui/UiInput.vue";
import UiSelect from "../../../../shared/ui/UiSelect.vue";
import { useWorkflowEditor } from "../../workflows/use-workflow-editor";
import type { WorkflowNodeKind } from "../../workflows/workflow-client";
import { WORKFLOW_NODE_KINDS } from "../../workflows/workflow-node-factory";

const props = defineProps<{ editor: ReturnType<typeof useWorkflowEditor> }>();
const view = defineModel<"graph" | "outline">("view", { required: true });
const { t } = useI18n();
const newNodeKind = shallowRef<WorkflowNodeKind>("action");
const draftOptions = computed(() =>
  props.editor.drafts.value.map((draft) => ({ value: draft.id, label: draft.definition.title })),
);
const nodeMessageKeys: Record<(typeof WORKFLOW_NODE_KINDS)[number], string> = {
  action: "spaces.assistant.workflowNode.action",
  condition: "spaces.assistant.workflowNode.condition",
  parallel: "spaces.assistant.workflowNode.parallel",
  join: "spaces.assistant.workflowNode.join",
  for_each: "spaces.assistant.workflowNode.forEach",
  iterate: "spaces.assistant.workflowNode.iterate",
  subworkflow: "spaces.assistant.workflowNode.subworkflow",
  human_input: "spaces.assistant.workflowNode.humanInput",
  review: "spaces.assistant.workflowNode.review",
  data_transform: "spaces.assistant.workflowNode.dataTransform",
  delay: "spaces.assistant.workflowNode.delay",
  external_tool: "spaces.assistant.workflowNode.externalTool",
  delegate_task: "spaces.assistant.workflowNode.delegateTask",
  publish: "spaces.assistant.workflowNode.publish",
};
const nodeOptions = computed(() =>
  WORKFLOW_NODE_KINDS.map((value) => ({ value, label: t(nodeMessageKeys[value]) })),
);
const templateOptions = computed(() =>
  props.editor.templates.value.map(({ id, name }) => ({ value: id, label: name })),
);
</script>

<template>
  <header class="space-y-2 border-b border-divider p-2.5">
    <div class="flex gap-1.5">
      <UiSelect
        v-model="editor.selectedId.value"
        class="min-w-0 flex-1"
        :label="t('spaces.assistant.workflows')"
        :options="draftOptions"
      />
      <UiButton size="sm" @click="editor.create">{{ t("common.create") }}</UiButton>
      <UiButton size="sm" variant="ghost" :disabled="!editor.draft.value" @click="editor.duplicate">{{
        t("common.duplicate")
      }}</UiButton>
      <UiButton size="sm" variant="ghost" @click="editor.extractHistory">{{
        t("spaces.assistant.fromHistory")
      }}</UiButton>
      <UiButton
        size="sm"
        variant="dangerSubtle"
        :disabled="!editor.draft.value"
        @click="editor.removeDraft"
        >{{ t("common.delete") }}</UiButton
      >
    </div>
    <UiInput
      v-if="editor.draft.value"
      v-model="editor.draft.value.definition.title"
      :placeholder="t('spaces.assistant.workflowTitle')"
    />
    <div class="flex flex-wrap gap-1.5">
      <UiButton size="sm" :variant="view === 'graph' ? 'secondary' : 'ghost'" @click="view = 'graph'">{{
        t("spaces.assistant.graph")
      }}</UiButton>
      <UiButton size="sm" :variant="view === 'outline' ? 'secondary' : 'ghost'" @click="view = 'outline'">{{
        t("spaces.assistant.outline")
      }}</UiButton>
      <UiButton
        size="sm"
        variant="ghost"
        :disabled="editor.undoStack.value.length === 0"
        @click="editor.undo"
        >{{ t("common.undo") }}</UiButton
      >
      <UiButton
        size="sm"
        variant="ghost"
        :disabled="editor.redoStack.value.length === 0"
        @click="editor.redo"
        >{{ t("common.redo") }}</UiButton
      >
      <UiSelect
        v-model="newNodeKind"
        class="ml-auto w-32"
        size="sm"
        :label="t('spaces.assistant.addStep')"
        :options="nodeOptions"
      />
      <UiButton size="sm" @click="editor.addNode(newNodeKind)">{{ t("spaces.assistant.addStep") }}</UiButton>
    </div>
    <div class="flex gap-1.5">
      <UiSelect
        v-model="editor.selectedTemplateId.value"
        class="min-w-0 flex-1"
        size="sm"
        :label="t('spaces.assistant.personalTemplates')"
        :options="templateOptions"
      />
      <UiButton
        size="sm"
        variant="ghost"
        :disabled="!editor.selectedTemplateId.value"
        @click="editor.useTemplate"
        >{{ t("spaces.assistant.useTemplate") }}</UiButton
      >
      <UiButton size="sm" variant="ghost" :disabled="!editor.draft.value" @click="editor.saveTemplate">{{
        t("spaces.assistant.saveTemplate")
      }}</UiButton>
      <UiButton
        size="sm"
        variant="dangerSubtle"
        :disabled="!editor.selectedTemplateId.value"
        @click="editor.removeTemplate"
        >{{ t("common.delete") }}</UiButton
      >
    </div>
  </header>
</template>
