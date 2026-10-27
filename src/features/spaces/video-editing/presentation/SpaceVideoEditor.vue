<script setup lang="ts">
import { toRef, useTemplateRef } from "vue";

import SpaceEditorProblem from "../../document-editing/presentation/SpaceEditorProblem.vue";
import type { SpaceEditorTab } from "../../workspace/workbench-tab";
import SpaceVideoEditorFallback from "./SpaceVideoEditorFallback.vue";
import SpaceVideoEditorHeader from "./SpaceVideoEditorHeader.vue";
import SpaceVideoUnsavedDialog from "./SpaceVideoUnsavedDialog.vue";
import SpaceVideoWorkbench from "./SpaceVideoWorkbench.vue";
import { useVideoEditor } from "./use-video-editor";

const props = defineProps<{ tab: SpaceEditorTab; active: boolean }>();
const editorRoot = useTemplateRef<HTMLElement>("editorRoot");
const editor = useVideoEditor(props.tab, toRef(props, "active"), editorRoot);
const { session, actions, project } = editor;
</script>

<template>
  <section
    ref="editorRoot"
    class="absolute inset-0 z-60 flex min-h-0 flex-col bg-surface-canvas"
    data-video-editor
  >
    <SpaceVideoEditorHeader :editor="editor" />
    <SpaceEditorProblem
      v-if="session.error.value"
      :error="session.error.value"
      @discard="actions.requestClose"
    />
    <SpaceVideoWorkbench v-else-if="project" :editor="editor" />
    <SpaceVideoEditorFallback v-else :editor="editor" />
  </section>
  <SpaceVideoUnsavedDialog
    v-model:open="actions.closeDialogOpen.value"
    :saving="actions.saving.value"
    :can-save="!session.error.value"
    @cancel="actions.cancelLeave"
    @discard="actions.confirmDiscard"
    @save="actions.confirmSave"
  />
</template>
