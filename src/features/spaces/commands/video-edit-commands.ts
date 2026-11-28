import { shallowRef } from "vue";

import { i18n } from "../../../foundation/i18n/instance";
import { availableWhen } from "../../../shared/contracts/action-availability";
import type { AppCommand } from "../../../shared/contracts/app-command";

export interface VideoEditCommandPort {
  canUndo(): boolean;
  canRedo(): boolean;
  canDelete(): boolean;
  canSplit(): boolean;
  canDuplicate(): boolean;
  undo(): void;
  redo(): void;
  remove(): void;
  split(): void;
  duplicate(): void;
  addText(): void;
  togglePlayback(): void;
  step(direction: -1 | 1, frames?: number): void;
  jump(edge: "start" | "end"): void;
  jumpCut(direction: -1 | 1): void;
  toggleSnapping(): void;
  zoomTimeline(factor: number): void;
  fitTimeline(): void;
  save(): Promise<void>;
}

const activeEditor = shallowRef<VideoEditCommandPort>();

export function registerVideoEditCommands(port: VideoEditCommandPort) {
  activeEditor.value = port;
  return () => {
    if (activeEditor.value === port) activeEditor.value = undefined;
  };
}

const editCmd = (spec: Omit<AppCommand, "category">): AppCommand => ({ category: "editing", ...spec });

export function videoEditCommands(): AppCommand[] {
  const t = i18n.global.t;
  return [
    editCmd({
      id: "video-edit.undo",
      title: t("common.undo"),
      shortcut: "$mod+Z",
      editingIntent: "undo",
      availability: () => availableWhen(!!activeEditor.value?.canUndo(), "nothing_to_undo"),
      run: () => activeEditor.value?.undo(),
    }),
    editCmd({
      id: "video-edit.redo",
      title: t("common.redo"),
      shortcut: "$mod+Shift+Z",
      editingIntent: "redo",
      availability: () => availableWhen(!!activeEditor.value?.canRedo(), "nothing_to_redo"),
      run: () => activeEditor.value?.redo(),
    }),
    editCmd({
      id: "video-edit.save",
      title: t("common.save"),
      shortcut: "$mod+S",
      run: () => activeEditor.value?.save(),
    }),
    editCmd({
      id: "video-edit.split",
      title: t("spaces.media.videoEditor.clip.split"),
      shortcut: "S",
      keybindings: ["S", "$mod+B"],
      availability: () => availableWhen(!!activeEditor.value?.canSplit(), "selection_not_eligible"),
      run: () => activeEditor.value?.split(),
    }),
    editCmd({
      id: "video-edit.delete",
      title: t("common.delete"),
      shortcut: "Backspace",
      keybindings: ["Backspace", "Delete"],
      availability: () => availableWhen(!!activeEditor.value?.canDelete(), "no_selection"),
      run: () => activeEditor.value?.remove(),
    }),
    editCmd({
      id: "video-edit.play",
      title: t("spaces.media.player.play"),
      shortcut: "Space",
      run: () => activeEditor.value?.togglePlayback(),
    }),
    editCmd({
      id: "video-edit.previous-step",
      title: t("spaces.media.videoEditor.playback.stepBackward"),
      shortcut: "ArrowLeft",
      allowRepeat: true,
      run: () => activeEditor.value?.step(-1),
    }),
    editCmd({
      id: "video-edit.next-step",
      title: t("spaces.media.videoEditor.playback.stepForward"),
      shortcut: "ArrowRight",
      allowRepeat: true,
      run: () => activeEditor.value?.step(1),
    }),
    editCmd({
      id: "video-edit.previous-step-ten",
      title: t("spaces.media.videoEditor.playback.stepBackTen"),
      shortcut: "Shift+ArrowLeft",
      allowRepeat: true,
      run: () => activeEditor.value?.step(-1, 10),
    }),
    editCmd({
      id: "video-edit.next-step-ten",
      title: t("spaces.media.videoEditor.playback.stepForwardTen"),
      shortcut: "Shift+ArrowRight",
      allowRepeat: true,
      run: () => activeEditor.value?.step(1, 10),
    }),
    editCmd({
      id: "video-edit.previous-cut",
      title: t("spaces.media.videoEditor.playback.previousCut"),
      shortcut: "ArrowUp",
      run: () => activeEditor.value?.jumpCut(-1),
    }),
    editCmd({
      id: "video-edit.next-cut",
      title: t("spaces.media.videoEditor.playback.nextCut"),
      shortcut: "ArrowDown",
      run: () => activeEditor.value?.jumpCut(1),
    }),
    editCmd({
      id: "video-edit.duplicate",
      title: t("spaces.media.videoEditor.clip.duplicate"),
      shortcut: "$mod+D",
      availability: () => availableWhen(!!activeEditor.value?.canDuplicate(), "no_selection"),
      run: () => activeEditor.value?.duplicate(),
    }),
    editCmd({
      id: "video-edit.add-text",
      title: t("spaces.media.videoEditor.text.add"),
      shortcut: "T",
      run: () => activeEditor.value?.addText(),
    }),
    editCmd({
      id: "video-edit.toggle-snapping",
      title: t("spaces.media.videoEditor.timeline.snapping"),
      shortcut: "N",
      run: () => activeEditor.value?.toggleSnapping(),
    }),
    editCmd({
      id: "video-edit.zoom-in",
      title: t("spaces.media.videoEditor.timeline.zoomIn"),
      shortcut: "$mod+Equal",
      run: () => activeEditor.value?.zoomTimeline(1.4),
    }),
    editCmd({
      id: "video-edit.zoom-out",
      title: t("spaces.media.videoEditor.timeline.zoomOut"),
      shortcut: "$mod+Minus",
      run: () => activeEditor.value?.zoomTimeline(1 / 1.4),
    }),
    editCmd({
      id: "video-edit.fit-timeline",
      title: t("spaces.media.videoEditor.timeline.fit"),
      shortcut: "Shift+Z",
      run: () => activeEditor.value?.fitTimeline(),
    }),
    editCmd({
      id: "video-edit.go-to-start",
      title: t("spaces.media.videoEditor.playback.goToStart"),
      shortcut: "Home",
      run: () => activeEditor.value?.jump("start"),
    }),
    editCmd({
      id: "video-edit.go-to-end",
      title: t("spaces.media.videoEditor.playback.goToEnd"),
      shortcut: "End",
      run: () => activeEditor.value?.jump("end"),
    }),
  ];
}
