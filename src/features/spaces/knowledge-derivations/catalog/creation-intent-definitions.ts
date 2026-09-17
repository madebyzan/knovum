import type {
  CreationIntentDecision,
  CreationIntentDefinition,
  CreationSelectionContext,
} from "./creation-intent-contract";

const AVAILABLE: CreationIntentDecision = { available: true };
const HIDDEN: CreationIntentDecision = { available: false };

function decideKnowledgeAction(context: CreationSelectionContext): CreationIntentDecision {
  if (context.hasKnowledgeSources) return AVAILABLE;
  return context.selectionCount > 0 ? { available: false, reason: "needs-indexed-knowledge" } : HIDDEN;
}

function decideKnowledgeVideo(context: CreationSelectionContext): CreationIntentDecision {
  const knowledge = decideKnowledgeAction(context);
  if (!knowledge.available) return knowledge;
  return context.imageCount > 10 ? { available: false, reason: "too-many-video-images" } : AVAILABLE;
}

export const creationIntentDefinitions = [
  {
    id: "grounded-document",
    kind: "atomic",
    labelKey: "spaces.creation.generateDocument",
    icon: "file-pen",
    decide: decideKnowledgeAction,
  },
  {
    id: "knowledge-visualization",
    kind: "atomic",
    labelKey: "spaces.creation.visualize",
    icon: "image",
    decide: decideKnowledgeAction,
  },
  {
    id: "docx-export",
    kind: "atomic",
    labelKey: "spaces.creation.exportDocx",
    icon: "file-output",
    decide: ({ canExportDocument }) => (canExportDocument ? AVAILABLE : HIDDEN),
  },
  {
    id: "pdf-export",
    kind: "atomic",
    labelKey: "spaces.creation.exportPdf",
    icon: "file-output",
    decide: ({ canExportDocument }) => (canExportDocument ? AVAILABLE : HIDDEN),
  },
  {
    id: "presentation",
    kind: "workflow",
    labelKey: "spaces.creation.createPresentation",
    icon: "file-output",
    decide: decideKnowledgeAction,
  },
  {
    id: "knowledge-video",
    kind: "workflow",
    labelKey: "spaces.creation.generateVideo",
    icon: "sparkles",
    decide: decideKnowledgeVideo,
  },
  {
    id: "social-content-kit",
    kind: "workflow",
    labelKey: "spaces.creation.socialContentKit",
    icon: "sparkles",
    decide: decideKnowledgeAction,
  },
  {
    id: "background-removal",
    kind: "atomic",
    labelKey: "spaces.creation.removeBackground",
    icon: "scan",
    decide: ({ canRemoveBackground }) => (canRemoveBackground ? AVAILABLE : HIDDEN),
  },
  {
    id: "video-with-subtitles",
    kind: "workflow",
    labelKey: "spaces.creation.videoWithSubtitlesWorkflow",
    icon: "captions",
    decide: ({ canExtractSubtitles }) => (canExtractSubtitles ? AVAILABLE : HIDDEN),
  },
  {
    id: "meeting-video-package",
    kind: "workflow",
    labelKey: "spaces.creation.meetingVideoPackage",
    icon: "sparkles",
    decide: ({ canCreateMeetingPackage }) => (canCreateMeetingPackage ? AVAILABLE : HIDDEN),
  },
  {
    id: "subtitle-burn-in",
    kind: "atomic",
    labelKey: "spaces.creation.createSubtitledVideo",
    icon: "captions",
    decide: ({ canBurnSubtitles }) => (canBurnSubtitles ? AVAILABLE : HIDDEN),
  },
  {
    id: "subtitle-extraction",
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
