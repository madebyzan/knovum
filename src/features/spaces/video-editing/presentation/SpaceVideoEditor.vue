<script setup lang="ts">
import { toRef } from "vue";

import SpaceEditorProblem from "../../document-editing/presentation/SpaceEditorProblem.vue";
import type { SpaceEditorTab } from "../../workspace/workbench-tab";
import SpaceVideoEditorFallback from "./SpaceVideoEditorFallback.vue";
import SpaceVideoEditorHeader from "./SpaceVideoEditorHeader.vue";
import SpaceVideoUnsavedDialog from "./SpaceVideoUnsavedDialog.vue";
import SpaceVideoWorkbench from "./SpaceVideoWorkbench.vue";
import { useVideoEditor } from "./use-video-editor";

const props = defineProps<{ tab: SpaceEditorTab; active: boolean }>();
const editor = useVideoEditor(props.tab, toRef(props, "active"));
const { session, actions, project, editorRoot } = editor;
</script>

<template>
  <section
    v-if="active"
    :ref="(value) => (editorRoot = value as HTMLElement | null)"
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
