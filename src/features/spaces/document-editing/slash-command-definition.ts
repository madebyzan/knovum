import type { ChainedCommands, Editor, Range } from "@tiptap/core";
import type { Component } from "vue";

export type SlashCommandGroup = "ai" | "blocks" | "lists" | "insert";

export interface SlashCommandCallbacks {
  openAiPrompt: (range: Range) => void;
  continueWithAi: (range: Range) => void;
  summarizeWithAi: (range: Range) => void;
  openMathPrompt: (kind: "inline" | "block", range: Range) => void;
}

export interface SlashCommandItem {
  readonly id: string;
  readonly group: SlashCommandGroup;
  readonly labelKey: string;
  readonly descriptionKey: string;
  readonly shortcut?: string | undefined;
  readonly keywords: readonly string[];
  readonly icon: Component;
  readonly run: (editor: Editor, range: Range, callbacks: SlashCommandCallbacks) => void;
}

export const SLASH_GROUP_LABEL_KEY: Readonly<Record<SlashCommandGroup, string>> = {
  ai: "spaces.richtext.slash.groups.ai",
  blocks: "spaces.richtext.slash.groups.blocks",
  lists: "spaces.richtext.slash.groups.lists",
  insert: "spaces.richtext.slash.groups.insert",
};

type SlashCommandAction = (chain: ChainedCommands, range: Range, callbacks: SlashCommandCallbacks) => void;

export type SlashCommandSpec = readonly [
  id: string,
  group: SlashCommandGroup,
  labelKey: string,
  descriptionKey: string,
  shortcut: string | undefined,
  keywords: readonly string[],
  icon: Component,
  action: SlashCommandAction,
];

export function buildSlashCommands(specs: readonly SlashCommandSpec[]): readonly SlashCommandItem[] {
  return specs.map(([id, group, labelKey, descriptionKey, shortcut, keywords, icon, action]) => ({
    id,
    group,
    labelKey,
    descriptionKey,
    shortcut,
    keywords,
    icon,
    run: (editor, range, callbacks) => {
      action(editor.chain().focus().deleteRange(range), range, callbacks);
    },
  }));
}
