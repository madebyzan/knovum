import { ListCollapse, ListPlus, ScanText, SpellCheck2 } from "@lucide/vue";
import type { Component } from "vue";

import type { TranslationTarget } from "../../../foundation/ai/translation";
import type { EditorTransformAction } from "../../../shared/contracts/generated/EditorTransformAction";
import { ContinueTextIcon, PolishTextIcon } from "./editor-icons";

export type RunningAction = EditorTransformAction | `translate:${TranslationTarget}`;

export const TONE_ACTION_KEYS: readonly (readonly [EditorTransformAction, string])[] = [
  ["tone_academic", "spaces.richtext.ai.tones.academic"],
  ["tone_business", "spaces.richtext.ai.tones.business"],
  ["tone_casual", "spaces.richtext.ai.tones.casual"],
  ["tone_confident", "spaces.richtext.ai.tones.confident"],
  ["tone_creative", "spaces.richtext.ai.tones.creative"],
  ["tone_friendly", "spaces.richtext.ai.tones.friendly"],
  ["tone_humorous", "spaces.richtext.ai.tones.humorous"],
  ["tone_informative", "spaces.richtext.ai.tones.informative"],
  ["tone_inspirational", "spaces.richtext.ai.tones.inspirational"],
  ["tone_objective", "spaces.richtext.ai.tones.objective"],
  ["tone_persuasive", "spaces.richtext.ai.tones.persuasive"],
];

export interface AiInlineCommand {
  readonly action: EditorTransformAction;
  readonly labelKey: string;
  readonly icon: Component;
  readonly needsSelection: boolean;
}

export const AI_INLINE_COMMANDS: readonly AiInlineCommand[] = [
  {
    action: "improve",
    labelKey: "spaces.richtext.ai.actions.improve",
    icon: PolishTextIcon,
    needsSelection: true,
  },
  {
    action: "fix_grammar",
    labelKey: "spaces.richtext.ai.actions.fixGrammar",
    icon: SpellCheck2,
    needsSelection: true,
  },
  {
    action: "reduce",
    labelKey: "spaces.richtext.ai.actions.reduce",
    icon: ListCollapse,
    needsSelection: true,
  },
  { action: "extend", labelKey: "spaces.richtext.ai.actions.extend", icon: ListPlus, needsSelection: true },
  {
    action: "simplify",
    labelKey: "spaces.richtext.ai.actions.simplify",
    icon: ScanText,
    needsSelection: true,
  },
  {
    action: "complete",
    labelKey: "spaces.richtext.ai.actions.complete",
    icon: ContinueTextIcon,
    needsSelection: false,
  },
  {
    action: "summarize",
    labelKey: "spaces.richtext.ai.actions.summarize",
    icon: ListCollapse,
    needsSelection: true,
  },
];
