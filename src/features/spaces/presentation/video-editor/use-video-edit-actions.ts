import { computed, onMounted, onUnmounted, shallowRef, type Ref } from "vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";

import { getErrorMessage } from "../../../../foundation/errors/error-format";
import { systemSansFamily } from "../../../../foundation/typography/system-font";
import { registerVideoEditCommands } from "../../commands/video-edit-commands";
import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import type { SpaceVideoPayload } from "../../model/nodes/node-payload";
import type { useSpaceStore } from "../../session/space-store";
import {
  addTextOverlay,
  locateTimelineTime,
  splitClip,
  type VideoEditProject,
} from "../../video-editing/project";
import { renderVideoEdit } from "../../video-editing/render";
import type { useVideoEditHistory } from "./use-video-edit-history";
import type { useVideoEditSelection } from "./use-video-edit-selection";
import type { useVideoProjectPlayback } from "./use-video-project-playback";

interface Options {
  store: ReturnType<typeof useSpaceStore>;
  node: Ref<SpaceCanvasNode | undefined>;
  data: Ref<SpaceVideoPayload | undefined>;
  project: Ref<VideoEditProject | undefined>;
  video: Ref<HTMLVideoElement | null>;
  history: ReturnType<typeof useVideoEditHistory>;
  selection: ReturnType<typeof useVideoEditSelection>;
  playback: ReturnType<typeof useVideoProjectPlayback>;
}

export function useVideoEditActions(options: Options) {
  const { store, node, data, project, video, history, selection, playback } = options;
  const { t } = useI18n();
  const closeDialogOpen = shallowRef(false);
  const saving = shallowRef(false);
  const rendering = shallowRef(false);
  const canDelete = computed(
    () =>
      !!selection.selectedTextId.value ||
      (!!selection.selectedClipId.value && (project.value?.videoTrack.clips.length ?? 0) > 1),
  );

  async function save(): Promise<boolean> {
    if (saving.value || !node.value || !data.value || !project.value) return false;
    // Captured before the await so edits landing mid-save stay dirty.
    const snapshot = project.value;
    const after: SpaceVideoPayload = { ...data.value, videoEditProject: project.value };
    saving.value = true;
    try {
      await store.updateNodeContents([{ id: node.value.id, before: data.value, after }]);
      history.markSaved(snapshot);
      toast.success(t("spaces.media.videoEditor.project.saved"));
      return true;
    } catch (error) {
      toast.error(getErrorMessage(error));
      return false;
    } finally {
      saving.value = false;
    }
  }

  function requestClose() {
    if (saving.value) return;
    if (history.dirty.value) closeDialogOpen.value = true;
    else store.closeVideoEditor();
  }

  function closeOrDismiss() {
    if (saving.value) return;
    if (closeDialogOpen.value) closeDialogOpen.value = false;
    else requestClose();
  }

  async function saveAndClose() {
    if ((await save()) && !history.dirty.value) store.closeVideoEditor();
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
    const before = project.value;
    selection.removeSelection();
    if (project.value === before) return;
    const after = project.value;
    project.value = before;
    history.commit(after);
  }

  function step(milliseconds: number) {
    video.value?.pause();
    playback.seek(playback.timelineMs.value + milliseconds);
  }

  function togglePlayback() {
    void playback.toggle().catch((error) => toast.error(getErrorMessage(error)));
  }

  async function render() {
    if (rendering.value || !node.value || node.value.data.kind !== "video" || !project.value) return;
    rendering.value = true;
    try {
      if (history.dirty.value && !(await save())) return;
      await renderVideoEdit(store, { ...node.value, data: node.value.data }, project.value);
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
    closeDialogOpen,
    remove,
    render,
    rendering,
    requestClose,
    save,
    saving,
    saveAndClose,
    split,
    step,
    togglePlayback,
  };
}
