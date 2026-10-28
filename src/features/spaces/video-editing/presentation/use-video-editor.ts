import { computed, onScopeDispose, shallowRef, watch, type Ref, type ShallowRef } from "vue";

import { getErrorMessage } from "../../../../foundation/errors/error-format";
import type { SpaceEditorTab } from "../../workspace/workbench-tab";
import { isVideoMaterial } from "../material";
import { createPreviewPlayer } from "../playback/preview-player";
import type { VideoEditProject } from "../project";
import { createFilmstripCache } from "../timeline/filmstrip";
import { useTimelineGestures } from "../timeline/use-timeline-gestures";
import { useTimelineView } from "../timeline/use-timeline-view";
import { createWaveformCache } from "../timeline/waveform";
import { useVideoSessionStore } from "../video-session-store";
import { useVideoEditActions } from "./use-video-edit-actions";
import { useVideoMaterials } from "./use-video-materials";
import { useVideoProjectBootstrap } from "./use-video-project-bootstrap";
import { useVideoSelection } from "./use-video-selection";

/**
 * Composition root of the video editor: one session, one preview player, one timeline view, and the
 * editing commands that operate on them.
 */
export function useVideoEditor(
  tab: SpaceEditorTab,
  active: Ref<boolean>,
  editorRoot: Readonly<ShallowRef<HTMLElement | null>>,
) {
  const session = useVideoSessionStore().sessions.get(tab.id);
  if (!session) throw new Error("Video session is unavailable");
  const data = computed(() => session.node.value.data);
  const project = session.project;
  const draft = shallowRef<VideoEditProject>();
  const visibleProject = computed(() => draft.value ?? project.value);
  const previewError = shallowRef<string>();

  const materials = useVideoMaterials(tab.spaceId, project, (next) => session.history.commit(next));
  const player = createPreviewPlayer({
    spaceId: tab.spaceId,
    project: () => visibleProject.value,
    materials: () => materials.available.value,
    onFailed: (error) => {
      player.pause();
      previewError.value = getErrorMessage(error);
    },
  });
  const filmstrips = createFilmstripCache({
    videos: player.videos,
    suspended: () => player.playing.value,
  });
  const waveforms = createWaveformCache({ spaceId: tab.spaceId, audible: player.media.audible });
  const selection = useVideoSelection({
    project,
    selection: session.selection,
    seek: player.seek,
    pause: player.pause,
    playheadMs: player.positionMs,
  });
  const timeline = useTimelineView({
    project: () => visibleProject.value,
    subscribe: player.subscribe,
  });
  const gestures = useTimelineGestures({
    project: () => project.value!,
    timeAt: (event) => timeline.timeAt(event),
    pxPerSecond: () => timeline.pxPerSecond.value,
    playheadMs: player.positionMs,
    commit: (next, mergeKey) => session.history.commit(next, mergeKey),
    seek: player.seek,
    select: (value) => selection.select(value, { reveal: false }),
    pause: player.pause,
  });
  watch(gestures.draft, (value) => (draft.value = value));

  const actions = useVideoEditActions({
    session,
    active,
    project,
    commit: (next, mergeKey) => session.history.commit(next, mergeKey),
    selection,
    playheadMs: player.positionMs,
    seek: player.seek,
    pause: player.pause,
    toggle: player.toggle,
    focusText: () => editorRoot.value?.querySelector<HTMLTextAreaElement>("#video-edit-text")?.select(),
  });

  const bootstrap = useVideoProjectBootstrap({
    node: computed(() => (isVideoMaterial(session.node.value) ? session.node.value : undefined)),
    project,
    obsolete: session.obsolete,
    active,
    onCreated: (created) => {
      project.value = created;
      session.history.reset(created);
    },
  });

  watch(
    active,
    (isActive) => {
      if (isActive) {
        player.seek(session.timelineMs.value);
        player.start();
      } else player.stop();
    },
    { immediate: true },
  );
  watch(player.playing, (playing) => {
    if (playing) return;
    session.timelineMs.value = player.positionMs();
    filmstrips.resume();
  });
  watch([project, materials.available], () => player.revise());
  watch(
    () => data.value.sourceRevision,
    () => (previewError.value = undefined),
  );
  onScopeDispose(() => {
    session.timelineMs.value = player.positionMs();
    player.dispose();
    filmstrips.dispose();
    waveforms.dispose();
  });

  /** Sources still waiting for a proxy because the WebView cannot decode their codec. */
  const converting = computed(() => visibleProject.value?.assets.some(player.videos.converting) ?? false);

  const renderable = computed(
    () =>
      !session.error.value &&
      !!project.value &&
      !previewError.value &&
      !actions.rendering.value &&
      [...project.value.videoTrack.clips, ...project.value.audioTrack.clips].every(({ assetId }) =>
        materials.available.value.has(assetId),
      ),
  );

  return {
    session,
    data,
    project,
    visibleProject,
    player,
    timeline,
    gestures,
    filmstrips,
    waveforms,
    selection,
    materials,
    actions,
    bootstrap,
    history: session.history,
    previewError,
    converting,
    retryPreview() {
      previewError.value = undefined;
      player.retry();
    },
    renderable,
    commit: (next: VideoEditProject, mergeKey?: string) => session.history.commit(next, mergeKey),
  };
}

export type VideoEditor = ReturnType<typeof useVideoEditor>;
