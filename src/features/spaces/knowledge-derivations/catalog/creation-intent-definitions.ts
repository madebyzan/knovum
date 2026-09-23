import type {
  CreationIntentDecision,
  CreationIntentDefinition,
  CreationSelectionContext,
} from "./creation-intent-contract";

const AVAILABLE: CreationIntentDecision = { state: "ready" };
const HIDDEN: CreationIntentDecision = { state: "hidden" };

function decideKnowledgeIntent(context: CreationSelectionContext): CreationIntentDecision {
  if (context.hasKnowledgeSources) return AVAILABLE;
  return context.selectionCount > 0 ? { state: "blocked", reason: "needs-indexed-knowledge" } : HIDDEN;
}

function decideVideoGeneration(context: CreationSelectionContext): CreationIntentDecision {
  const knowledge = decideKnowledgeIntent(context);
  if (knowledge.state !== "ready") return knowledge;
  return context.imageCount > 10 ? { state: "blocked", reason: "too-many-video-images" } : AVAILABLE;
}

export const creationIntentDefinitions = [
  {
    id: "draft-document",
    kind: "atomic",
    labelKey: "spaces.creation.generateDocument",
    icon: "file-pen",
    decide: decideKnowledgeIntent,
  },
  {
    id: "generate-visualization",
    kind: "atomic",
    labelKey: "spaces.creation.visualize",
    icon: "image",
    decide: decideKnowledgeIntent,
  },
  {
    id: "export-document-as-docx",
    kind: "atomic",
    labelKey: "spaces.creation.exportDocx",
    icon: "file-output",
    decide: ({ canExportDocument }) => (canExportDocument ? AVAILABLE : HIDDEN),
  },
  {
    id: "export-document-as-pdf",
    kind: "atomic",
    labelKey: "spaces.creation.exportPdf",
    icon: "file-output",
    decide: ({ canExportDocument }) => (canExportDocument ? AVAILABLE : HIDDEN),
  },
  {
    id: "presentation-production",
    kind: "workflow",
    labelKey: "spaces.creation.createPresentation",
    icon: "file-output",
    decide: decideKnowledgeIntent,
  },
  {
    id: "video-production",
    kind: "workflow",
    labelKey: "spaces.creation.generateVideo",
    icon: "sparkles",
    decide: decideVideoGeneration,
  },
  {
    id: "social-content-package",
    kind: "workflow",
    labelKey: "spaces.creation.socialContentPackage",
    icon: "sparkles",
    decide: decideKnowledgeIntent,
  },
  {
    id: "remove-image-background",
    kind: "atomic",
    labelKey: "spaces.creation.removeBackground",
    icon: "scan",
    decide: ({ canRemoveBackground }) => (canRemoveBackground ? AVAILABLE : HIDDEN),
  },
  {
    id: "subtitled-video-production",
    kind: "workflow",
    labelKey: "spaces.creation.subtitledVideoProduction",
    icon: "captions",
    decide: ({ canExtractSubtitles }) => (canExtractSubtitles ? AVAILABLE : HIDDEN),
  },
  {
    id: "meeting-content-package",
    kind: "workflow",
    labelKey: "spaces.creation.meetingContentPackage",
    icon: "sparkles",
    decide: ({ canCreateMeetingPackage }) => (canCreateMeetingPackage ? AVAILABLE : HIDDEN),
  },
  {
    id: "render-subtitled-video",
    kind: "atomic",
    labelKey: "spaces.creation.createSubtitledVideo",
    icon: "captions",
    decide: ({ canBurnSubtitles }) => (canBurnSubtitles ? AVAILABLE : HIDDEN),
  },
  {
    id: "extract-subtitles",
    kind: "atomic",
    labelKey: "spaces.creation.createSubtitleResource",
    icon: "captions",
    decide: ({ canExtractSubtitles }) => (canExtractSubtitles ? AVAILABLE : HIDDEN),
  },
] as const satisfies readonly CreationIntentDefinition[];

export type CreationIntentDefinitionEntry = (typeof creationIntentDefinitions)[number];
export type CreationIntentId = CreationIntentDefinitionEntry["id"];
export type CreationIntentDecisions = Record<CreationIntentId, CreationIntentDecision>;

export function decideCreationIntents(context: CreationSelectionContext): CreationIntentDecisions {
  return Object.fromEntries(
    creationIntentDefinitions.map((definition) => [definition.id, definition.decide(context)]),
  ) as CreationIntentDecisions;
}
