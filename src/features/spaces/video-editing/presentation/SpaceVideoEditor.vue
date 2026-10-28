<script setup lang="ts">
import { toRef, useTemplateRef } from "vue";

import SpaceEditorProblem from "../../document-editing/presentation/SpaceEditorProblem.vue";
import { useSpaceStore } from "../../session/space-store";
import type { SpaceEditorTab } from "../../workspace/workbench-tab";
import SpaceVideoEditorFallback from "./SpaceVideoEditorFallback.vue";
import SpaceVideoEditorHeader from "./SpaceVideoEditorHeader.vue";
import SpaceVideoWorkbench from "./SpaceVideoWorkbench.vue";
import { useVideoEditor } from "./use-video-editor";

const props = defineProps<{ tab: SpaceEditorTab; active: boolean }>();
const store = useSpaceStore();
const editorRoot = useTemplateRef<HTMLElement>("editorRoot");
const editor = useVideoEditor(props.tab, toRef(props, "active"), editorRoot);
const { session, project } = editor;
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
      @discard="store.discardUnavailableEditorTab(tab.id)"
    />
    <SpaceVideoWorkbench v-else-if="project" :editor="editor" />
    <SpaceVideoEditorFallback v-else :editor="editor" />
  </section>
</template>
