import { computed, nextTick, shallowRef, type Ref } from "vue";

import { getErrorMessage } from "../../../../foundation/errors/error-format";
import type { SpaceEditorTab } from "../../workspace/work-items";
import { useVideoEditProxy } from "../use-video-edit-proxy";
import { useVideoSessionStore } from "../video-session-store";
import { useVideoEditActions } from "./use-video-edit-actions";
import { useVideoEditProject } from "./use-video-edit-project";
import { useVideoEditSelection } from "./use-video-edit-selection";
import { useVideoEditorInspector } from "./use-video-editor-inspector";
import { useVideoMaterials } from "./use-video-materials";
import { useVideoProjectPlayback } from "./use-video-project-playback";

export function useVideoEditor(tab: SpaceEditorTab, active: Ref<boolean>) {
  const session = useVideoSessionStore().sessions.get(tab.id);
  if (!session) throw new Error("Video session is unavailable");
  const data = computed(() => session.node.value.data);
  const proxy = useVideoEditProxy(
    computed(() => tab.nodeId),
    computed(() => data.value.sourceRevision),
    computed(() => tab.spaceId),
    active,
  );
  const project = session.project;
  const editorRoot = shallowRef<HTMLElement | null>(null);
  const metadata = useVideoEditProject(project, data, tab.nodeId);
  const previewError = shallowRef<string>();
  const trimDraft = shallowRef<typeof project.value>();
  const visibleProject = computed(() => trimDraft.value ?? project.value);
  const selection = useVideoEditSelection(
    project,
    session.selectedClipId,
    session.selectedTextId,
    session.selectedAudioId,
  );
  const playback = useVideoProjectPlayback(visibleProject, active, session.timelineMs);
  const inspector = useVideoEditorInspector({ project, editorRoot, selection, playback });
  const history = session.history;
  const actions = useVideoEditActions({
    session,
    active,
    node: session.node,
    project,
    history,
    selection,
    playback,
  });
  const materials = useVideoMaterials(tab.spaceId, project, history.commit);
  const editorError = computed(() => proxy.error.value ?? metadata.metadataError.value);
  function onMetadataReady(event: Event) {
    metadata.initializeFromMetadata(event);
    if (project.value) history.reset(data.value.videoEditProject ?? project.value);
  }
  function failPreview(error: unknown) {
    playback.pause();
    previewError.value = getErrorMessage(error);
  }
  async function onAddText() {
    actions.addText();
    await nextTick();
    const input = editorRoot.value?.querySelector<HTMLTextAreaElement>("#video-edit-text");
    input?.focus();
    input?.select();
  }
  return {
    session,
    data,
    project,
    visibleProject,
    trimDraft,
    editorRoot,
    timelineZoom: session.timelineZoom,
    selection,
    playback,
    inspector,
    history,
    actions,
    materials,
    editorError,
    previewError,
    failPreview,
    proxyPath: proxy.path,
    prepareError: proxy.error,
    retryProxy: () => {
      metadata.metadataError.value = undefined;
      proxy.retry();
    },
    onMetadataReady,
    onAddText,
    failMetadata: (message: string) => {
      metadata.metadataError.value = message;
    },
  };
}

export type VideoEditor = ReturnType<typeof useVideoEditor>;
