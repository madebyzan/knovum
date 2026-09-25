import { shallowRef } from "vue";

import { availableWhen } from "../../../shared/contracts/action-availability";
import type { AppCommand } from "../../../shared/contracts/app-command";

export interface VideoEditCommandPort {
  canUndo(): boolean;
  canRedo(): boolean;
  canDelete(): boolean;
  undo(): void;
  redo(): void;
  remove(): void;
  split(): void;
  togglePlayback(): void;
  step(milliseconds: number): void;
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

export function videoEditCommands(t: (key: string) => string): AppCommand[] {
  const port = () => activeEditor.value;
  return [
    {
      id: "video-edit.undo",
      title: t("common.undo"),
      shortcut: "$mod+Z",
      category: "editing",
      availability: () => availableWhen(!!port()?.canUndo(), "nothing_to_undo"),
      run: () => port()?.undo(),
    },
    {
      id: "video-edit.redo",
      title: t("common.redo"),
      shortcut: "$mod+Shift+Z",
      category: "editing",
      availability: () => availableWhen(!!port()?.canRedo(), "nothing_to_redo"),
      run: () => port()?.redo(),
    },
    {
      id: "video-edit.save",
      title: t("common.save"),
      shortcut: "$mod+S",
      category: "editing",
      run: () => port()?.save(),
    },
    {
      id: "video-edit.split",
      title: t("spaces.media.split"),
      shortcut: "S",
      category: "editing",
      run: () => port()?.split(),
    },
    {
      id: "video-edit.delete",
      title: t("common.delete"),
      shortcut: "Backspace",
      keybindings: ["Backspace", "Delete"],
      category: "editing",
      availability: () => availableWhen(!!port()?.canDelete(), "no_selection"),
      run: () => port()?.remove(),
    },
    {
      id: "video-edit.play",
      title: t("spaces.media.play"),
      shortcut: "Space",
      category: "editing",
      run: () => port()?.togglePlayback(),
    },
    {
      id: "video-edit.previous-step",
      title: t("spaces.media.previousStep"),
      shortcut: "ArrowLeft",
      category: "editing",
      run: () => port()?.step(-100),
    },
    {
      id: "video-edit.next-step",
      title: t("spaces.media.nextStep"),
      shortcut: "ArrowRight",
      category: "editing",
      run: () => port()?.step(100),
    },
    {
      id: "video-edit.close",
      title: t("common.close"),
      shortcut: "Escape",
      category: "editing",
      run: () => port()?.close(),
    },
  ];
}
