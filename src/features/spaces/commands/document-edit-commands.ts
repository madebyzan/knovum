import { shallowRef } from "vue";

import { i18n } from "../../../foundation/i18n/instance";
import { availableWhen } from "../../../shared/contracts/action-availability";
import type { AppCommand } from "../../../shared/contracts/app-command";

export interface DocumentEditCommandPort {
  canUndo(): boolean;
  canRedo(): boolean;
  undo(): void;
  redo(): void;
  find(): void;
  focusOutline(): void;
  hasOutline(): boolean;
  save(): Promise<void>;
}

const activeEditor = shallowRef<DocumentEditCommandPort>();

export function registerDocumentEditCommands(port: DocumentEditCommandPort) {
  activeEditor.value = port;
  return () => {
    if (activeEditor.value === port) activeEditor.value = undefined;
  };
}

export function documentEditCommands(): AppCommand[] {
  const port = activeEditor.value;
  if (!port) return [];
  const t = i18n.global.t;
  return [
    {
      id: "document.undo",
      title: t("common.undo"),
      shortcut: "$mod+Z",
      category: "editing",
      allowInEditable: true,
      availability: () => availableWhen(port.canUndo(), "nothing_to_undo"),
      run: port.undo,
    },
    {
      id: "document.redo",
      title: t("common.redo"),
      shortcut: "$mod+Shift+Z",
      category: "editing",
      allowInEditable: true,
      availability: () => availableWhen(port.canRedo(), "nothing_to_redo"),
      run: port.redo,
    },
    {
      id: "document.find",
      title: t("spaces.richtext.find.title"),
      shortcut: "$mod+F",
      category: "editing",
      allowInEditable: true,
      run: port.find,
    },
    {
      id: "document.focus-outline",
      availability: () => (port.hasOutline() ? { state: "ready" } : { state: "hidden" }),
      title: t("spaces.sidebar.editorOutline.title"),
      shortcut: "$mod+Shift+O",
      category: "navigation",
      allowInEditable: true,
      run: port.focusOutline,
    },
    {
      id: "document.save",
      title: t("common.save"),
      shortcut: "$mod+S",
      category: "editing",
      allowInEditable: true,
      run: port.save,
    },
  ];
}
