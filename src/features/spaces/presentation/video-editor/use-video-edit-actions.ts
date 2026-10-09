import { computed, onMounted, onUnmounted, shallowRef, type Ref } from "vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";

import { getErrorMessage } from "../../../../foundation/errors/error-format";
import { systemSansFamily } from "../../../../foundation/typography/system-font";
import { registerVideoEditCommands } from "../../commands/video-edit-commands";
import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import type { SpaceVideoPayload } from "../../model/nodes/node-payload";
import {
  addTextOverlay,
  locateTimelineTime,
  splitClip,
  type VideoEditProject,
} from "../../video-editing/project";
import { saveVideoEditProject, type UpdateNodeContents } from "../../video-editing/save";
import type { useVideoEditHistory } from "./use-video-edit-history";
import { useVideoEditLeave } from "./use-video-edit-leave";
import type { useVideoEditSelection } from "./use-video-edit-selection";
import type { useVideoProjectPlayback } from "./use-video-project-playback";

interface Options {
  updateNodeContents: UpdateNodeContents;
  close: () => void;
  renderProject: (source: SpaceCanvasNode<SpaceVideoPayload>, project: VideoEditProject) => Promise<void>;
  node: Ref<SpaceCanvasNode | undefined>;
  data: Ref<SpaceVideoPayload | undefined>;
  project: Ref<VideoEditProject | undefined>;
  video: Ref<HTMLVideoElement | null>;
  history: ReturnType<typeof useVideoEditHistory>;
  selection: ReturnType<typeof useVideoEditSelection>;
  playback: ReturnType<typeof useVideoProjectPlayback>;
}

export function useVideoEditActions(options: Options) {
  const {
    updateNodeContents,
    close,
    renderProject,
    node,
    data,
    project,
    video,
    history,
    selection,
    playback,
  } = options;
  const { t } = useI18n();
  const saving = shallowRef(false);
  const rendering = shallowRef(false);
  let savePromise: Promise<boolean> | undefined;
  const canDelete = computed(
    () =>
      !!selection.selectedTextId.value ||
      (!!selection.selectedClipId.value && (project.value?.videoTrack.clips.length ?? 0) > 1),
  );

  function save(): Promise<boolean> {
    if (savePromise) return savePromise;
    const currentNode = node.value;
    const currentPayload = data.value;
    const currentProject = project.value;
    if (!currentNode || !currentPayload || !currentProject) return Promise.resolve(false);
    // Captured before the await so edits landing mid-save stay dirty.
    const snapshot = currentProject;
    saving.value = true;
    savePromise = saveVideoEditProject(updateNodeContents, currentNode, currentPayload, currentProject)
      .then(() => {
        history.markSaved(snapshot);
        toast.success(t("spaces.media.videoEditor.project.saved"));
        return true;
      })
      .catch((error) => {
        toast.error(getErrorMessage(error));
        return false;
      })
      .finally(() => {
        saving.value = false;
        savePromise = undefined;
      });
    return savePromise;
  }

  const leave = useVideoEditLeave({
    dirty: history.dirty,
    saving,
    save,
    discard: close,
    close,
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
    if (rendering.value || !sourceNode || sourceNode.type !== "video" || !project.value) return;
    rendering.value = true;
    try {
      if (history.dirty.value && !(await save())) return;
      await renderProject(sourceNode, project.value);
    } catch (error) {
      toast.error(`${t("spaces.media.videoEditor.render.failed")}: ${getErrorMessage(error)}`);
    } finally {
      rendering.value = false;
    }
  }

  let unregister: (() => void) | undefined;
  onMounted(() => {
    unregister = registerVideoEditCommands({
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
    });
  });
  onUnmounted(() => unregister?.());

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
