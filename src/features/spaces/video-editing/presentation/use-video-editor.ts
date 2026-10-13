import { computed, onScopeDispose, shallowRef, watch, type Ref } from "vue";

import { getErrorMessage } from "../../../../foundation/errors/error-format";
import type { SpaceEditorTab } from "../../workspace/workbench-tab";
import { createPreviewEngine } from "../engine/preview-engine";
import { isVideoMaterial } from "../material";
import type { VideoEditProject } from "../project";
import { createFilmstripCache } from "../timeline/filmstrip";
import { useTimelineGestures } from "../timeline/use-timeline-gestures";
import { useTimelineView } from "../timeline/use-timeline-view";
import { createWaveformCache } from "../timeline/waveform";
import { useVideoSessionStore } from "../video-session-store";
import { useVideoProjectBootstrap } from "./use-video-bootstrap";
import { useVideoEditActions } from "./use-video-edit-actions";
import { useVideoMaterials } from "./use-video-materials";
import { useVideoSelection } from "./use-video-selection";

/**
 * Composition root of the video editor: one session, one preview engine, one timeline view, and the
 * editing commands that operate on them.
 */
export function useVideoEditor(tab: SpaceEditorTab, active: Ref<boolean>) {
  const session = useVideoSessionStore().sessions.get(tab.id);
  if (!session) throw new Error("Video session is unavailable");
  const data = computed(() => session.node.value.data);
  const project = session.project;
  const draft = shallowRef<VideoEditProject>();
  const visibleProject = computed(() => draft.value ?? project.value);
  const previewError = shallowRef<string>();
  const editorRoot = shallowRef<HTMLElement | null>(null);

  const materials = useVideoMaterials(tab.spaceId, project, (next) => session.history.commit(next));
  const engine = createPreviewEngine({
    spaceId: tab.spaceId,
    project: () => visibleProject.value,
    materials: () => materials.available.value,
    onFailed: (error) => {
      engine.pause();
      previewError.value = getErrorMessage(error);
    },
  });
  const filmstrips = createFilmstripCache({
    sources: engine.sources,
    suspended: () => engine.playing.value,
  });
  const waveforms = createWaveformCache({ sources: engine.sources, audio: engine.audio });
  const selection = useVideoSelection({
    project,
    selection: session.selection,
    seek: engine.seek,
    pause: engine.pause,
    playheadMs: engine.positionMs,
  });
  const timeline = useTimelineView({
    project: () => visibleProject.value,
    subscribe: engine.subscribe,
  });
  const gestures = useTimelineGestures({
    project: () => project.value!,
    timeAt: (event) => timeline.timeAt(event),
    pxPerSecond: () => timeline.pxPerSecond.value,
    playheadMs: engine.positionMs,
    commit: (next, mergeKey) => session.history.commit(next, mergeKey),
    seek: engine.seek,
    select: (value) => selection.select(value, { reveal: false }),
    pause: engine.pause,
  });
  watch(gestures.draft, (value) => (draft.value = value));

  const actions = useVideoEditActions({
    session,
    active,
    project,
    commit: (next, mergeKey) => session.history.commit(next, mergeKey),
    selection,
    playheadMs: engine.positionMs,
    seek: engine.seek,
    pause: engine.pause,
    toggle: engine.toggle,
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
        engine.seek(session.timelineMs.value);
        engine.start();
      } else engine.stop();
    },
    { immediate: true },
  );
  watch(engine.playing, (playing) => {
    if (playing) return;
    session.timelineMs.value = engine.positionMs();
    filmstrips.resume();
  });
  watch([project, materials.available], () => engine.invalidate());
  watch(
    () => data.value.sourceRevision,
    () => (previewError.value = undefined),
  );
  onScopeDispose(() => {
    session.timelineMs.value = engine.positionMs();
    engine.dispose();
    filmstrips.dispose();
    waveforms.dispose();
  });

  const renderable = computed(
    () =>
      !session.error.value &&
      !!project.value &&
      !previewError.value &&
      !actions.saving.value &&
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
    editorRoot,
    engine,
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
    renderable,
    commit: (next: VideoEditProject, mergeKey?: string) => session.history.commit(next, mergeKey),
  };
}

export type VideoEditor = ReturnType<typeof useVideoEditor>;
