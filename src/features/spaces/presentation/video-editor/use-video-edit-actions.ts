import { computed, watch, type Ref } from "vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";

import { getErrorMessage } from "../../../../foundation/errors/error-format";
import { systemSansFamily } from "../../../../foundation/typography/system-font";
import { registerVideoEditCommands } from "../../commands/video-edit-commands";
import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import type { SpaceVideoPayload } from "../../model/nodes/node-payload";
import { useSpaceStore } from "../../session/space-store";
import {
  addTextOverlay,
  locateTimelineTime,
  splitClip,
  type VideoEditProject,
} from "../../video-editing/project";
import { renderVideoEdit } from "../../video-editing/render";
import type { useVideoEditHistory } from "../../video-editing/use-video-edit-history";
import { useVideoProjectSave } from "../../video-editing/use-video-project-save";
import type { VideoSession } from "../../video-editing/video-session-store";
import { useVideoEditLeave } from "./use-video-edit-leave";
import type { useVideoEditSelection } from "./use-video-edit-selection";
import type { useVideoProjectPlayback } from "./use-video-project-playback";

interface VideoEditActionsOptions {
  session: VideoSession;
  active: Ref<boolean>;
  node: Ref<SpaceCanvasNode<SpaceVideoPayload> | undefined>;
  project: Ref<VideoEditProject | undefined>;
  video: Ref<HTMLVideoElement | null>;
  history: ReturnType<typeof useVideoEditHistory>;
  selection: ReturnType<typeof useVideoEditSelection>;
  playback: ReturnType<typeof useVideoProjectPlayback>;
}

export function useVideoEditActions(options: VideoEditActionsOptions) {
  const { node, project, video, history, selection, playback } = options;
  const store = useSpaceStore();
  const { t } = useI18n();
  const saving = options.session.saving;
  const rendering = options.session.rendering;
  const save = useVideoProjectSave(options.session);
  const canDelete = computed(
    () =>
      !!selection.selectedTextId.value ||
      (!!selection.selectedClipId.value && (project.value?.videoTrack.clips.length ?? 0) > 1),
  );

  const leave = useVideoEditLeave({
    tab: options.session.tab,
    dirty: history.dirty,
    saving,
    save,
    close: () => store.closeEditorTab(options.session.tab.id),
    discard: history.discard,
  });
  function closeOrDismiss() {
    if (saving.value) return;
    if (leave.closeDialogOpen.value) leave.cancel();
    else leave.requestClose();
  }

  function split() {
    if (!project.value) return;
    const next = splitClip(project.value, playback.timelineMs.value);
    history.commit(next);
    const right = locateTimelineTime(next, playback.timelineMs.value);
    if (right) selection.selectClip(right.clip.id);
  }

  function addText() {
    if (!project.value) return;
    const next = addTextOverlay(project.value, playback.timelineMs.value, systemSansFamily());
    history.commit(next);
    const added = next.textTrack.items.at(-1);
    if (added) selection.selectText(added.id);
  }

  function remove() {
    if (!project.value || !canDelete.value) return;
    const next = selection.removeSelection();
    if (next) history.commit(next);
  }

  function step(milliseconds: number) {
    video.value?.pause();
    playback.seek(playback.timelineMs.value + milliseconds);
  }

  function togglePlayback() {
    void playback.toggle().catch((error) => toast.error(getErrorMessage(error)));
  }

  async function render() {
    const sourceNode = node.value;
    const renderProject = project.value;
    const tab = options.session.tab;
    if (rendering.value || !sourceNode || !renderProject) return;
    rendering.value = true;
    try {
      if (history.dirty.value && !(await save())) return;
      if (options.session.error.value) return;
      await renderVideoEdit(tab.spaceId, sourceNode, renderProject);
    } catch (error) {
      toast.error(`${t("spaces.media.videoEditor.render.failed")}: ${getErrorMessage(error)}`);
    } finally {
      rendering.value = false;
    }
  }

  watch(
    [options.active, options.session.error],
    ([active, error], _previous, cleanup) => {
      if (!active || error) return;
      cleanup(
        registerVideoEditCommands({
          canUndo: () => history.canUndo.value,
          canRedo: () => history.canRedo.value,
          canDelete: () => canDelete.value,
          undo: history.undo,
          redo: history.redo,
          remove,
          split,
          togglePlayback,
          step,
          save: async () => {
            await save();
          },
          close: closeOrDismiss,
        }),
      );
    },
    { immediate: true },
  );

  return {
    addText,
    canDelete,
    cancelLeave: leave.cancel,
    closeDialogOpen: leave.closeDialogOpen,
    confirmDiscard: leave.discard,
    confirmSave: leave.save,
    remove,
    render,
    rendering,
    requestClose: leave.requestClose,
    save,
    saving,
    split,
    step,
    togglePlayback,
  };
}
