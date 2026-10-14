import { computed, nextTick, watch, type Ref } from "vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";

import { getErrorMessage } from "../../../../foundation/errors/error-format";
import { systemSansFamily } from "../../../../foundation/typography/system-font";
import { registerVideoEditCommands } from "../../commands/video-edit-commands";
import { useSpaceStore } from "../../session/space-store";
import { duplicateClip, splitClip } from "../edits/arrange";
import { addTextOverlay } from "../edits/text";
import { defaultTextStyle, type VideoEditProject } from "../project";
import { renderVideoEdit } from "../render";
import { canSplitAt, lastFrameMs, stepFrame } from "../timing";
import { useVideoProjectSave } from "../use-video-project-save";
import type { VideoSession } from "../video-session-store";
import { useVideoEditLeave } from "./use-video-edit-leave";
import type { VideoSelectionController } from "./use-video-selection";

interface ActionOptions {
  session: VideoSession;
  active: Ref<boolean>;
  project: Ref<VideoEditProject | undefined>;
  commit: (project: VideoEditProject, mergeKey?: string) => void;
  selection: VideoSelectionController;
  playheadMs: () => number;
  seek: (timeMs: number) => void;
  pause: () => void;
  toggle: () => void;
  focusText: () => void;
}

/** Editing commands shared by the toolbar, the context menus and the command palette. */
export function useVideoEditActions(options: ActionOptions) {
  const { session, project, selection } = options;
  const store = useSpaceStore();
  const { t } = useI18n();
  const save = useVideoProjectSave(session);
  const canSplit = computed(() => !!project.value && canSplitAt(project.value, options.playheadMs()));

  const leave = useVideoEditLeave({
    tab: session.tab,
    dirty: session.history.dirty,
    saving: session.saving,
    save,
    close: () => store.closeTab(session.tab.id),
    discard: session.history.discard,
  });

  function split() {
    const current = project.value;
    if (!current) return;
    const next = splitClip(current, options.playheadMs());
    if (next === current) return;
    options.commit(next);
    const tail = next.videoTrack.clips.find(
      (clip) => clip.timelineStartMs === Math.round(options.playheadMs()),
    );
    if (tail) selection.select({ kind: "clip", id: tail.id }, { reveal: false });
  }

  function duplicate() {
    const current = project.value;
    const clip = selection.clip.value;
    if (!current || !clip) return;
    options.commit(duplicateClip(current, clip.id));
  }

  async function addText() {
    const current = project.value;
    if (!current) return;
    const next = addTextOverlay(
      current,
      options.playheadMs(),
      defaultTextStyle(systemSansFamily()),
      t("spaces.media.videoEditor.text.default"),
    );
    options.commit(next);
    const added = next.textTrack.items.at(-1);
    if (added) selection.select({ kind: "text", id: added.id }, { reveal: false });
    await nextTick();
    options.focusText();
  }

  function remove() {
    const next = selection.removeSelected();
    if (next) options.commit(next);
  }

  function step(direction: -1 | 1) {
    if (!project.value) return;
    options.pause();
    options.seek(stepFrame(project.value, options.playheadMs(), direction));
  }

  function jump(edge: "start" | "end") {
    if (!project.value) return;
    options.pause();
    options.seek(edge === "start" ? 0 : lastFrameMs(project.value));
  }

  async function render() {
    const current = project.value;
    if (session.rendering.value || !current) return;
    session.rendering.value = true;
    try {
      if (session.history.dirty.value && !(await save())) return;
      if (session.error.value) return;
      await renderVideoEdit(session.tab.spaceId, session.node.value, current);
    } catch (error) {
      toast.error(`${t("spaces.media.videoEditor.render.failed")}: ${getErrorMessage(error)}`);
    } finally {
      session.rendering.value = false;
    }
  }

  watch(
    [options.active, session.error],
    ([isActive, error], _previous, cleanup) => {
      if (!isActive || error) return;
      cleanup(
        registerVideoEditCommands({
          canUndo: () => session.history.canUndo.value,
          canRedo: () => session.history.canRedo.value,
          canDelete: () => selection.canDelete.value,
          canSplit: () => canSplit.value,
          canDuplicate: () => !!selection.clip.value,
          undo: session.history.undo,
          redo: session.history.redo,
          remove,
          split,
          duplicate,
          addText: () => void addText(),
          togglePlayback: options.toggle,
          step,
          jump,
          save: async () => {
            await save();
          },
        }),
      );
    },
    { immediate: true },
  );

  return {
    canSplit,
    canDelete: selection.canDelete,
    addText,
    duplicate,
    remove,
    render,
    rendering: session.rendering,
    save,
    saving: session.saving,
    split,
    step,
    jump,
    closeDialogOpen: leave.closeDialogOpen,
    cancelLeave: leave.cancel,
    confirmDiscard: leave.discard,
    confirmSave: leave.save,
    requestClose: leave.requestClose,
  };
}
