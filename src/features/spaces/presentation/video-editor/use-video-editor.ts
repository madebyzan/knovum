import { computed, nextTick, shallowRef, watch, type Ref } from "vue";

import { useVideoEditProxy } from "../../video-editing/use-video-edit-proxy";
import { useVideoSessionStore } from "../../video-editing/video-session-store";
import type { SpaceEditorTab } from "../../workspace/work-items";
import { useVideoEditActions } from "./use-video-edit-actions";
import { useVideoEditPreviewSource } from "./use-video-edit-preview-source";
import { useVideoEditProject } from "./use-video-edit-project";
import { useVideoEditSelection } from "./use-video-edit-selection";
import { useVideoEditorInspector } from "./use-video-editor-inspector";
import { useVideoEditorMedia } from "./use-video-editor-media";
import { useVideoProjectPlayback } from "./use-video-project-playback";

export function useVideoEditor(tab: SpaceEditorTab, active: Ref<boolean>) {
  const session = useVideoSessionStore().sessions.get(tab.id);
  if (!session) throw new Error("Video session is unavailable");
  const node = session.node;
  const data = computed(() => node.value.data);
  const {
    path: proxyPath,
    error: prepareError,
    retry: retryProxy,
  } = useVideoEditProxy(
    computed(() => node.value?.id),
    computed(() => data.value?.sourceRevision),
    computed(() => tab.spaceId),
    active,
  );
  const { project, metadataError, initializeFromMetadata } = useVideoEditProject(
    session.project,
    data,
    proxyPath,
  );
  const video = shallowRef<HTMLVideoElement | null>(null);
  const editorRoot = shallowRef<HTMLElement | null>(null);
  const timelineZoom = session.timelineZoom;
  const selection = useVideoEditSelection(project, session.selectedClipId, session.selectedTextId);
  const playback = useVideoProjectPlayback(video, project, session.timelineMs);
  const {
    visible: inspectorVisible,
    selectText: onSelectText,
    selectClip: onSelectClip,
    close: closeInspector,
  } = useVideoEditorInspector({ project, video, editorRoot, selection, playback });
  const {
    previewPath,
    error: previewError,
    onMetadataError,
  } = useVideoEditPreviewSource({
    data,
    proxyPath,
    playing: playback.playing,
  });
  const editorError = computed(
    () => prepareError.value ?? metadataError.value ?? previewError.value ?? playback.audioError.value,
  );
  const history = session.history;
  const actions = useVideoEditActions({
    session,
    active,
    node,
    project,
    video,
    history,
    selection,
    playback,
  });
  const { sourceDurationMs, onMetadataReady, onPreviewReady } = useVideoEditorMedia({
    data,
    project,
    video,
    history,
    playback,
    initializeFromMetadata,
  });

  watch(project, () => playback.seek(playback.timelineMs.value));
  watch(
    active,
    (visible) => {
      if (!visible) {
        video.value?.pause();
        playback.playing.value = false;
      }
    },
    { flush: "sync" },
  );

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
    proxyPath,
    prepareError,
    retryProxy,
    project,
    video,
    editorRoot,
    timelineZoom,
    selection,
    playback,
    inspectorVisible,
    onSelectText,
    onSelectClip,
    closeInspector,
    previewPath,
    editorError,
    history,
    actions,
    sourceDurationMs,
    onMetadataReady,
    onPreviewReady,
    onMetadataError,
    onAddText,
  };
}
