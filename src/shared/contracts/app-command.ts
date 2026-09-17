import type { Component } from "vue";

export type CommandCategory = "general" | "navigation" | "canvas" | "editing";

export interface CommandShortcutListing {
  groupId: string;
  title: string;
  shortcut: string;
}

export interface AppCommand {
  id: string;
  title: string;
  description?: string;
  icon?: Component;
  shortcut?: string;
  keybindings?: readonly string[];
  shortcutListing?: CommandShortcutListing;
  category?: CommandCategory;
  appMenuShortcut?: true;
  allowInEditable?: true;
  isEnabled?: () => boolean;
  run: () => void | Promise<void>;
}
