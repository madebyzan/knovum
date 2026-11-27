import { ListCollapse } from "@lucide/vue";
import type { Editor } from "@tiptap/core";
import { search } from "fast-fuzzy";

import { i18n } from "../../../foundation/i18n/instance";
import { ContinueTextIcon, PolishTextIcon } from "./editor-icons";
import {
  buildSlashCommands,
  type SlashCommandCallbacks,
  type SlashCommandItem,
  type SlashCommandSpec,
} from "./slash-command-definition";
import { SLASH_BLOCK_SPECS } from "./slash-command-specs";
import { insertAndEditMath, type MathEditTarget } from "./tiptap-math";
import type { SpaceTiptapWritingTools } from "./use-space-tiptap-writing-tools";

const SLASH_AI_SPECS: readonly SlashCommandSpec[] = [
  [
    "ai-ask",
    "ai",
    "spaces.richtext.slash.items.askAi.label",
    "spaces.richtext.slash.items.askAi.description",
    "⌘J",
    ["ai", "ask", "knovum", "prompt", "write", "draft", "写作", "指令", "提问"],
    PolishTextIcon,
    (chain, range, callbacks) => (chain.run(), callbacks.openAiPrompt(range)),
  ],
  [
    "ai-continue",
    "ai",
    "spaces.richtext.slash.items.continue.label",
    "spaces.richtext.slash.items.continue.description",
    undefined,
    ["continue", "complete", "ai", "续写", "补全"],
    ContinueTextIcon,
    (chain, range, callbacks) => (chain.run(), callbacks.continueWithAi(range)),
  ],
  [
    "ai-summarize",
    "ai",
    "spaces.richtext.slash.items.summarize.label",
    "spaces.richtext.slash.items.summarize.description",
    undefined,
    ["summarize", "summary", "ai", "总结", "摘要"],
    ListCollapse,
    (chain, range, callbacks) => (chain.run(), callbacks.summarizeWithAi(range)),
  ],
];

const SLASH_COMMANDS: readonly SlashCommandItem[] = buildSlashCommands([
  ...SLASH_AI_SPECS,
  ...SLASH_BLOCK_SPECS,
]);

export function createSpaceSlashCallbacks(
  getEditor: () => Editor | undefined,
  writing: SpaceTiptapWritingTools,
  openMath: (target: MathEditTarget) => void,
): SlashCommandCallbacks {
  return {
    openAiPrompt: (range) => writing.openPrompt({ from: range.from, to: range.from }),
    continueWithAi: (range) => void writing.run("complete", { range: { from: range.from, to: range.from } }),
    summarizeWithAi: () => {
      const size = getEditor()?.state.doc.content.size;
      if (size !== undefined) void writing.run("summarize", { range: { from: 0, to: size } });
    },
    openMathPrompt: (kind, range) => {
      const editor = getEditor();
      if (editor) insertAndEditMath(editor, kind, openMath, { range });
    },
  };
}

export function filterSlashCommands(
  query: string,
  items: readonly SlashCommandItem[] = SLASH_COMMANDS,
): readonly SlashCommandItem[] {
  const normalized = query.trim();
  if (!normalized) return items;
  return search(normalized, [...items], {
    keySelector: (item) => [
      i18n.global.t(item.labelKey),
      ...item.keywords,
      i18n.global.t(item.descriptionKey),
    ],
    threshold: 0.6,
  });
}
