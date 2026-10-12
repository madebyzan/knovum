import { shallowRef } from "vue";

import { i18n } from "../../../foundation/i18n/instance";
import { availableWhen } from "../../../shared/contracts/action-availability";
import type { AppCommand } from "../../../shared/contracts/app-command";

export interface VideoEditCommandPort {
  canUndo(): boolean;
  canRedo(): boolean;
  canDelete(): boolean;
  canSplit(): boolean;
  undo(): void;
  redo(): void;
  remove(): void;
  split(): void;
  togglePlayback(): void;
  step(direction: -1 | 1): void;
  save(): Promise<void>;
  close(): void;
}

const activeEditor = shallowRef<VideoEditCommandPort>();

export function registerVideoEditCommands(port: VideoEditCommandPort) {
  activeEditor.value = port;
  return () => {
    if (activeEditor.value === port) activeEditor.value = undefined;
  };
}

export function videoEditCommands(): AppCommand[] {
  const t = i18n.global.t;

  return [
    {
      id: "video-edit.undo",
      title: t("common.undo"),
      shortcut: "$mod+Z",
      category: "editing",
      availability: () => availableWhen(!!activeEditor.value?.canUndo(), "nothing_to_undo"),
      run: () => activeEditor.value?.undo(),
    },
    {
      id: "video-edit.redo",
      title: t("common.redo"),
      shortcut: "$mod+Shift+Z",
      category: "editing",
      availability: () => availableWhen(!!activeEditor.value?.canRedo(), "nothing_to_redo"),
      run: () => activeEditor.value?.redo(),
    },
    {
      id: "video-edit.save",
      title: t("common.save"),
      shortcut: "$mod+S",
      category: "editing",
      run: () => activeEditor.value?.save(),
    },
    {
      id: "video-edit.split",
      title: t("spaces.media.videoEditor.clip.split"),
      shortcut: "S",
      category: "editing",
      availability: () => availableWhen(!!activeEditor.value?.canSplit(), "selection_not_eligible"),
      run: () => activeEditor.value?.split(),
    },
    {
      id: "video-edit.delete",
      title: t("common.delete"),
      shortcut: "Backspace",
      keybindings: ["Backspace", "Delete"],
      category: "editing",
      availability: () => availableWhen(!!activeEditor.value?.canDelete(), "no_selection"),
      run: () => activeEditor.value?.remove(),
    },
    {
      id: "video-edit.play",
      title: t("spaces.media.player.play"),
      shortcut: "Space",
      category: "editing",
      run: () => activeEditor.value?.togglePlayback(),
    },
    {
      id: "video-edit.previous-step",
      title: t("spaces.media.videoEditor.playback.stepBackward"),
      shortcut: "ArrowLeft",
      category: "editing",
      run: () => activeEditor.value?.step(-1),
    },
    {
      id: "video-edit.next-step",
      title: t("spaces.media.videoEditor.playback.stepForward"),
      shortcut: "ArrowRight",
      category: "editing",
      run: () => activeEditor.value?.step(1),
    },
    {
      id: "video-edit.close",
      title: t("common.close"),
      shortcut: "Escape",
      category: "editing",
      run: () => activeEditor.value?.close(),
    },
  ];
}
