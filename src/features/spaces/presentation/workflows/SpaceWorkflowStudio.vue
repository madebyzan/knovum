<script setup lang="ts">
import { storeToRefs } from "pinia";
import { shallowRef } from "vue";
import { useI18n } from "vue-i18n";

import { useSpaceStore } from "../../session/space-store";
import { useWorkflowEditor } from "../../workflows/use-workflow-editor";
import SpaceWorkflowCanvas from "./SpaceWorkflowCanvas.vue";
import SpaceWorkflowExecutionPanel from "./SpaceWorkflowExecutionPanel.vue";
import SpaceWorkflowOutline from "./SpaceWorkflowOutline.vue";
import SpaceWorkflowToolbar from "./SpaceWorkflowToolbar.vue";

const { t } = useI18n();
const { activeSpaceId } = storeToRefs(useSpaceStore());
const editor = useWorkflowEditor(activeSpaceId);
const view = shallowRef<"graph" | "outline">("graph");
</script>

<template>
  <section class="flex h-full min-h-0 flex-col bg-panel">
    <SpaceWorkflowToolbar v-model:view="view" :editor="editor" />
    <div v-if="editor.draft.value" class="min-h-0 flex-1 overflow-auto">
      <SpaceWorkflowCanvas v-if="view === 'graph'" v-model="editor.draft.value" class="h-full" />
      <SpaceWorkflowOutline
        v-else
        v-model="editor.draft.value"
        @remove="editor.removeNode"
        @connect="editor.connectNodes"
      />
    </div>
    <div v-else class="text-meta flex flex-1 items-center justify-center p-6 text-foreground-tertiary">
      {{ t("spaces.assistant.workflowEmpty") }}
    </div>
    <SpaceWorkflowExecutionPanel :editor="editor" />
  </section>
</template>
