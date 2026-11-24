import {
  Compass,
  FileText,
  Focus,
  Lightbulb,
  ListChecks,
  ListTodo,
  Map,
  Scale,
  Waypoints,
} from "@lucide/vue";
import type { Component } from "vue";

export interface ConversationStarter {
  readonly id: string;
  readonly icon: Component;
  readonly titleKey: string;
  readonly descriptionKey: string;
  readonly promptKey: string;
  readonly accent?: boolean;
  readonly attachSelection?: boolean;
  readonly params?: Readonly<Record<string, string | number>>;
}

type StarterEntry = readonly [string, Component, string, string, string];

function starter([id, icon, titleKey, descriptionKey, promptKey]: StarterEntry): ConversationStarter {
  return { id, icon, titleKey, descriptionKey, promptKey };
}

const POPULATED_STARTERS = {
  overview: starter([
    "overview",
    Map,
    "spaces.conversation.empty.overview",
    "spaces.conversation.empty.overviewDesc",
    "spaces.conversation.empty.overviewPrompt",
  ]),
  brief: starter([
    "brief",
    FileText,
    "spaces.conversation.empty.brief",
    "spaces.conversation.empty.briefDesc",
    "spaces.conversation.empty.briefPrompt",
  ]),
  compare: starter([
    "compare",
    Waypoints,
    "spaces.conversation.empty.compare",
    "spaces.conversation.empty.compareDesc",
    "spaces.conversation.empty.comparePrompt",
  ]),
  takeaways: starter([
    "takeaways",
    ListChecks,
    "spaces.conversation.empty.takeaways",
    "spaces.conversation.empty.takeawaysDesc",
    "spaces.conversation.empty.takeawaysPrompt",
  ]),
  gaps: starter([
    "gaps",
    Compass,
    "spaces.conversation.empty.gaps",
    "spaces.conversation.empty.gapsDesc",
    "spaces.conversation.empty.gapsPrompt",
  ]),
  actions: starter([
    "actions",
    ListTodo,
    "spaces.conversation.empty.actions",
    "spaces.conversation.empty.actionsDesc",
    "spaces.conversation.empty.actionsPrompt",
  ]),
};

const EMPTY_STARTERS: readonly ConversationStarter[] = [
  starter([
    "draft",
    FileText,
    "spaces.conversation.empty.draft",
    "spaces.conversation.empty.draftDesc",
    "spaces.conversation.empty.draftPrompt",
  ]),
  starter([
    "plan",
    Compass,
    "spaces.conversation.empty.plan",
    "spaces.conversation.empty.planDesc",
    "spaces.conversation.empty.planPrompt",
  ]),
  starter([
    "questions",
    Lightbulb,
    "spaces.conversation.empty.questions",
    "spaces.conversation.empty.questionsDesc",
    "spaces.conversation.empty.questionsPrompt",
  ]),
  starter([
    "decision",
    Scale,
    "spaces.conversation.empty.decision",
    "spaces.conversation.empty.decisionDesc",
    "spaces.conversation.empty.decisionPrompt",
  ]),
];

const SELECTION_SINGLE = starter([
  "selection-single",
  Focus,
  "spaces.conversation.empty.selectionSingle",
  "spaces.conversation.empty.selectionSingleDesc",
  "spaces.conversation.empty.selectionSinglePrompt",
]);

const SELECTION_MULTI = starter([
  "selection-multi",
  Focus,
  "spaces.conversation.empty.selectionMulti",
  "spaces.conversation.empty.selectionMultiDesc",
  "spaces.conversation.empty.selectionMultiPrompt",
]);

export function buildEmptyConversationStarters(input: {
  sourceCount: number;
  selectedTitles: readonly string[];
  maxCount?: number;
}): readonly ConversationStarter[] {
  const count = input.selectedTitles.length;
  const selection =
    count === 0
      ? undefined
      : {
          ...(count === 1 ? SELECTION_SINGLE : SELECTION_MULTI),
          accent: true,
          attachSelection: true,
          params: count === 1 ? { title: input.selectedTitles[0]! } : { count },
        };
  const base: readonly ConversationStarter[] =
    input.sourceCount > 0
      ? [
          POPULATED_STARTERS.overview,
          POPULATED_STARTERS.brief,
          input.sourceCount >= 2 ? POPULATED_STARTERS.compare : POPULATED_STARTERS.takeaways,
          POPULATED_STARTERS.gaps,
          POPULATED_STARTERS.actions,
        ]
      : EMPTY_STARTERS;
  return (selection ? [selection, ...base] : base).slice(0, input.maxCount ?? 5);
}
