import { getProductShortcuts } from "../../foundation/shortcuts/shortcut-catalog";
import type { AppCommand, CommandCategory } from "../../shared/contracts/app-command";

export interface ShortcutListing {
  id: string;
  title: string;
  shortcut: string;
  category: CommandCategory;
}

export const CATEGORY_ORDER: readonly CommandCategory[] = ["general", "navigation", "canvas", "editing"];

export const LOCALE_KEY_BY_CATEGORY: Readonly<Record<CommandCategory, string>> = {
  general: "shell.shortcuts.categories.general",
  navigation: "shell.shortcuts.categories.navigation",
  canvas: "shell.shortcuts.categories.canvas",
  editing: "shell.shortcuts.categories.editing",
};

function appendCommandShortcuts(
  commands: readonly AppCommand[],
  list: ShortcutListing[],
  seen: Set<string>,
  getEffective: (id: string, def?: string) => string | undefined,
) {
  for (const cmd of commands) {
    if (!cmd.shortcut) continue;
    const effective = getEffective(cmd.id, cmd.shortcut) ?? cmd.shortcut;
    if (cmd.shortcutListing) {
      if (!seen.has(cmd.shortcutListing.groupId)) {
        seen.add(cmd.shortcutListing.groupId);
        list.push({
          id: cmd.shortcutListing.groupId,
          title: cmd.shortcutListing.title,
          shortcut: effective,
          category: cmd.category ?? "general",
        });
      }
      continue;
    }
    seen.add(cmd.id);
    list.push({ id: cmd.id, title: cmd.title, shortcut: effective, category: cmd.category ?? "general" });
  }
}

function appendCatalogShortcuts(
  list: ShortcutListing[],
  seen: Set<string>,
  translate: (key: string) => string,
  getEffective: (id: string, def?: string) => string | undefined,
) {
  for (const item of getProductShortcuts()) {
    if (seen.has(item.id)) continue;
    seen.add(item.id);
    const effective = getEffective(item.id, item.defaultShortcut) ?? item.defaultShortcut;
    list.push({ id: item.id, title: translate(item.titleKey), shortcut: effective, category: item.category });
  }
}

export function collectShortcuts(
  commands: readonly AppCommand[],
  translate: (key: string) => string,
  getEffective: (id: string, def?: string) => string | undefined,
): ShortcutListing[] {
  const list: ShortcutListing[] = [];
  const seen = new Set<string>();
  appendCommandShortcuts(commands, list, seen, getEffective);
  appendCatalogShortcuts(list, seen, translate, getEffective);
  return list;
}
