import type { Component } from "vue";

import type { ActionAvailability } from "./action-availability";

export type CommandCategory = "general" | "navigation" | "canvas" | "editing";
export type CommandUnavailableReason =
  | "nothing_to_undo"
  | "nothing_to_redo"
  | "no_selection"
  | "active_node_operation"
  | "not_enough_scenes"
  | "selection_not_eligible";

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
  availability?: () => ActionAvailability<CommandUnavailableReason>;
  run: () => void | Promise<void>;
}
