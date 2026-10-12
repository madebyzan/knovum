<script setup lang="ts">
import { toRef } from "vue";

import SpaceEditorProblem from "../../document-editing/presentation/SpaceEditorProblem.vue";
import type { SpaceEditorTab } from "../../workspace/workbench-tab";
import SpaceVideoEditorHeader from "./SpaceVideoEditorHeader.vue";
import SpaceVideoEditorState from "./SpaceVideoEditorState.vue";
import SpaceVideoUnsavedDialog from "./SpaceVideoUnsavedDialog.vue";
import SpaceVideoWorkbench from "./SpaceVideoWorkbench.vue";
import { useVideoEditor } from "./use-video-editor";
const props = defineProps<{ tab: SpaceEditorTab; active: boolean }>();
const editor = useVideoEditor(props.tab, toRef(props, "active"));
const {
  session,
  data,
  project,
  editorRoot,
  history,
  actions,
  materials,
  editorError,
  proxyPath,
  prepareError,
  retryProxy,
  onMetadataReady,
  failMetadata,
} = editor;
</script>
<template>
  <section
    v-if="active"
    :ref="(value) => (editorRoot = value as HTMLElement | null)"
    class="absolute inset-0 z-60 flex min-h-0 flex-col bg-surface-canvas"
    data-video-editor
  >
    <SpaceVideoEditorHeader
      :title="data.title"
      :can-save="
        !session.error.value &&
        !!project &&
        history.dirty.value &&
        !actions.saving.value &&
        !actions.rendering.value
      "
      :can-render="
        !session.error.value &&
        !!project &&
        [...project.videoTrack.clips, ...project.audioTrack.clips].every(({ assetId }) =>
          materials.available.value.has(assetId),
        ) &&
        !editorError &&
        !actions.saving.value &&
        !actions.rendering.value
      "
      :preparing="!proxyPath && !prepareError"
      :dirty="history.dirty.value"
      @close="actions.requestClose"
      @save="actions.save()"
      @render="actions.render"
    />
    <SpaceEditorProblem
      v-if="session.error.value"
      :error="session.error.value"
      @discard="actions.requestClose"
    />
    <SpaceVideoWorkbench v-else-if="project && !editorError" :editor="editor" />
    <SpaceVideoEditorState
      v-else
      :data="data"
      :proxy-path="proxyPath"
      :error="editorError"
      :can-retry="!!prepareError || !!editorError"
      @metadata-ready="onMetadataReady"
      @metadata-error="failMetadata($t('spaces.media.videoEditor.preview.unavailable'))"
      @retry="retryProxy"
    />
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
