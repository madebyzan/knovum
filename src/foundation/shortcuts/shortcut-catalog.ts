import { platform } from "@tauri-apps/plugin-os";

import type { CommandCategory } from "../../shared/contracts/app-command";

export interface ShortcutItemDefinition {
  readonly id: string;
  readonly titleKey: string;
  readonly defaultShortcut: string;
  readonly category: CommandCategory;
  readonly keybindings?: readonly string[];
  readonly isGlobal?: boolean;
  readonly descriptionKey?: string;
}

function def(
  id: string,
  titleKey: string,
  defaultShortcut: string,
  category: CommandCategory,
): ShortcutItemDefinition {
  return { id, titleKey, defaultShortcut, category };
}

function withExtra(
  item: ShortcutItemDefinition,
  extra: { keybindings?: readonly string[]; isGlobal?: boolean; descriptionKey?: string },
): ShortcutItemDefinition {
  return { ...item, ...extra };
}

function isMacPlatform(): boolean {
  try {
    return platform() === "macos";
  } catch {
    return typeof navigator !== "undefined" && /Mac|iPod|iPhone|iPad/.test(navigator.platform);
  }
}

const GENERAL_SHORTCUTS: readonly ShortcutItemDefinition[] = [
  def("shell.command-palette", "shell.commands.title", "$mod+K", "general"),
  def("navigate.settings", "common.settings", "$mod+,", "general"),
  def("shell.shortcuts", "shell.shortcuts.title", "$mod+/", "general"),
  withExtra(
    def("dictation", "settings.shortcuts.dictationLabel", "CommandOrControl+Shift+Space", "general"),
    {
      isGlobal: true,
      descriptionKey: "settings.shortcuts.dictationDescription",
    },
  ),
];

const NAVIGATION_SHORTCUTS: readonly ShortcutItemDefinition[] = [
  def("view.toggle-sidebar", "shell.layout.toggleSidebar", "$mod+B", "navigation"),
  def("view.toggle-right-sidebar", "shell.layout.toggleRightSidebar", "$mod+Shift+B", "navigation"),
  def("spaces.new-space", "spaces.space.new", "$mod+T", "navigation"),
  def("spaces.close-tab", "spaces.space.tabs.close", "$mod+W", "navigation"),
  def("spaces.reopen-closed-tab", "spaces.space.tabs.reopen", "$mod+Shift+T", "navigation"),
  def("spaces.next-tab", "spaces.space.tabs.next", "Control+Tab", "navigation"),
  def("spaces.previous-tab", "spaces.space.tabs.previous", "Control+Shift+Tab", "navigation"),
  def("spaces.previous-scene", "spaces.scenes.previous", "Alt+[", "navigation"),
  def("spaces.next-scene", "spaces.scenes.next", "Alt+]", "navigation"),
  def("spaces.navigate-back", "spaces.space.back", "Alt+ArrowLeft", "navigation"),
  def("spaces.navigate-forward", "spaces.space.forward", "Alt+ArrowRight", "navigation"),
];

const CANVAS_SHORTCUTS: readonly ShortcutItemDefinition[] = [
  def("spaces.tool-select", "spaces.canvas.tools.select", "V", "canvas"),
  def("spaces.tool-pan", "spaces.canvas.shortcuts.panCanvas", "H", "canvas"),
  def("spaces.tool-note", "spaces.editor.note.new", "N", "canvas"),
  def("spaces.tool-text", "spaces.editor.text.new", "T", "canvas"),
  def("spaces.tool-section", "spaces.canvas.tools.section", "S", "canvas"),
  withExtra(def("spaces.zoom-in", "spaces.canvas.zoom.in", "$mod+=", "canvas"), {
    keybindings: ["$mod+Equal", "$mod+Shift+Equal"],
  }),
  def("spaces.zoom-out", "spaces.canvas.zoom.out", "$mod+-", "canvas"),
  def("spaces.zoom-reset", "spaces.canvas.shortcuts.zoomReset", "$mod+0", "canvas"),
  withExtra(def("spaces.zoom-to-fit", "spaces.canvas.zoom.toFitContent", "Shift+2", "canvas"), {
    keybindings: ["Shift+Digit2"],
  }),
  withExtra(def("spaces.zoom-to-selection", "spaces.canvas.zoom.toSelection", "Shift+1", "canvas"), {
    keybindings: ["Shift+Digit1"],
  }),
  def("spaces.cancel-canvas-interaction", "spaces.canvas.cancelInteraction", "Escape", "canvas"),
  def("spaces.ask", "spaces.conversation.evidence.ask", "$mod+Enter", "canvas"),
  def("spaces.find", "spaces.search.find.title", "$mod+F", "canvas"),
  def("spaces.find-all", "spaces.search.allSpaces.title", "$mod+Shift+F", "canvas"),
  def("spaces.focus-outline", "spaces.sidebar.outline.title", "$mod+Shift+O", "canvas"),
  def("spaces.select-all", "spaces.canvas.selection.selectAll", "$mod+A", "canvas"),
];

function getEditingShortcuts(): readonly ShortcutItemDefinition[] {
  const isMac = isMacPlatform();
  return [
    def("spaces.group-selection", "spaces.canvas.groups.create", "$mod+G", "editing"),
    def("spaces.dissolve-group", "spaces.canvas.groups.ungroup", "$mod+Shift+G", "editing"),
    def("spaces.rename-node", "common.rename", "F2", "editing"),
    def("spaces.toggle-selection-lock", "spaces.canvas.selection.toggleLock", "$mod+Shift+L", "editing"),
    def("spaces.duplicate", "common.duplicate", "$mod+D", "editing"),
    def(
      "spaces.delete-selection",
      "spaces.canvas.selection.delete",
      isMac ? "Backspace" : "Delete",
      "editing",
    ),
    def("spaces.undo", "common.undo", "$mod+Z", "editing"),
    def("spaces.redo", "common.redo", "$mod+Shift+Z", "editing"),
    def("spaces.cut", "common.cut", "$mod+X", "editing"),
    def("spaces.copy", "common.copy", "$mod+C", "editing"),
    def("spaces.paste", "common.paste", "$mod+V", "editing"),
  ];
}

export function getProductShortcuts(): readonly ShortcutItemDefinition[] {
  return [...GENERAL_SHORTCUTS, ...NAVIGATION_SHORTCUTS, ...CANVAS_SHORTCUTS, ...getEditingShortcuts()];
}
