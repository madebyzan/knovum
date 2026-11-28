import { computed, nextTick, watch, type Ref } from "vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";

import { getErrorMessage } from "../../../../foundation/errors/error-format";
import { systemSansFamily } from "../../../../foundation/typography/system-font";
import { registerVideoEditCommands } from "../../commands/video-edit-commands";
import { editPoints } from "../edits/commit";
import { duplicateSelectedItem } from "../edits/duplicate-selection";
import { canSplitTarget, splitTargetAt } from "../edits/split-selection";
import { addTextOverlay } from "../edits/text";
import { defaultTextStyle, type VideoEditProject } from "../project";
import { renderVideoEdit } from "../render";
import { frameDurationMs, lastFrameMs, stepFrame } from "../timing";
import { useVideoProjectSave } from "../use-video-project-save";
import type { VideoSession } from "../video-session-store";
import type { VideoSelectionController } from "./use-video-selection";

interface ActionOptions {
  session: VideoSession;
  active: Ref<boolean>;
  project: Ref<VideoEditProject | undefined>;
  commit: (project: VideoEditProject, mergeKey?: string) => void;
  selection: VideoSelectionController;
  playheadMs: () => number;
  seek: (timeMs: number) => void;
  reveal: (timeMs: number) => void;
  zoom: (factor: number) => void;
  fitTimeline: () => void;
  toggleSnapping: () => void;
  pause: () => void;
  toggle: () => void;
  focusText: () => void;
}

/** Editing commands shared by the toolbar, the context menus and the command palette. */
export function useVideoEditActions(options: ActionOptions) {
  const { session, project, selection } = options;
  const { t } = useI18n();
  const autosave = useVideoProjectSave(session, options.active);
  const canSplit = computed(
    () => !!project.value && canSplitTarget(project.value, selection.selection.value, options.playheadMs()),
  );
  const canDuplicate = computed(() =>
    Boolean(selection.clip.value || selection.audio.value || selection.text.value),
  );

  function seekAndReveal(timeMs: number) {
    options.seek(timeMs);
    options.reveal(timeMs);
  }

  function split() {
    const current = project.value;
    if (!current) return;
    const result = splitTargetAt(current, selection.selection.value, options.playheadMs());
    if (result.project === current) return;
    options.commit(result.project);
    if (result.nextSelection) selection.select(result.nextSelection, { reveal: false });
  }

  function duplicate() {
    const current = project.value;
    if (!current) return;
    const result = duplicateSelectedItem(current, selection.selection.value);
    if (result.project === current) return;
    options.commit(result.project);
    if (result.nextSelection) selection.select(result.nextSelection, { reveal: false });
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

  function step(direction: -1 | 1, frames = 1) {
    const current = project.value;
    if (!current) return;
    options.pause();
    const target =
      frames === 1
        ? stepFrame(current, options.playheadMs(), direction)
        : Math.max(
            0,
            Math.min(
              lastFrameMs(current),
              Math.round(options.playheadMs() + direction * frames * frameDurationMs(current)),
            ),
          );
    seekAndReveal(target);
  }

  function jump(edge: "start" | "end") {
    if (!project.value) return;
    options.pause();
    seekAndReveal(edge === "start" ? 0 : lastFrameMs(project.value));
  }

  function jumpCut(direction: -1 | 1) {
    const current = project.value;
    if (!current) return;
    options.pause();
    const now = options.playheadMs();
    const points = editPoints(current);
    const target =
      direction < 0
        ? (points.findLast((point) => point < now - 5) ?? 0)
        : (points.find((point) => point > now + 5) ?? lastFrameMs(current));
    seekAndReveal(Math.min(target, lastFrameMs(current)));
  }

  async function render() {
    const current = project.value;
    if (session.rendering.value || !current) return;
    session.rendering.value = true;
    try {
      await autosave.persist();
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
          canDuplicate: () => canDuplicate.value,
          undo: session.history.undo,
          redo: session.history.redo,
          remove,
          split,
          duplicate,
          addText: () => void addText(),
          togglePlayback: options.toggle,
          step,
          jump,
          jumpCut,
          toggleSnapping: options.toggleSnapping,
          zoomTimeline: options.zoom,
          fitTimeline: options.fitTimeline,
          save: autosave.persist,
        }),
      );
    },
    { immediate: true },
  );

  return {
    canSplit,
    canDuplicate,
    canDelete: selection.canDelete,
    addText,
    duplicate,
    remove,
    render,
    rendering: session.rendering,
    saveStatus: autosave.status,
    split,
    step,
    jump,
    jumpCut,
  };
}
