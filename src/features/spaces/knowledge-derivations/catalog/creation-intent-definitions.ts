import type {
  CreationIntentDecision,
  CreationIntentDefinition,
  CreationSelectionContext,
} from "./creation-intent";

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
    effect: "derive",
    labelKey: "spaces.creation.document.generate",
    icon: "file-pen",
    decide: decideKnowledgeIntent,
  },
  {
    id: "generate-visualization",
    kind: "atomic",
    effect: "derive",
    labelKey: "spaces.creation.image.visualize",
    icon: "image",
    decide: decideKnowledgeIntent,
  },
  {
    id: "export-document-as-docx",
    kind: "atomic",
    effect: "export",
    labelKey: "spaces.creation.document.export.docx",
    icon: "file-output",
    decide: ({ canExportDocument }) => (canExportDocument ? AVAILABLE : HIDDEN),
  },
  {
    id: "export-document-as-pdf",
    kind: "atomic",
    effect: "export",
    labelKey: "spaces.creation.document.export.pdf",
    icon: "file-output",
    decide: ({ canExportDocument }) => (canExportDocument ? AVAILABLE : HIDDEN),
  },
  {
    id: "export-document-as-txt",
    kind: "atomic",
    effect: "export",
    labelKey: "spaces.creation.document.export.txt",
    icon: "file-output",
    decide: ({ canExportDocument }) => (canExportDocument ? AVAILABLE : HIDDEN),
  },
  {
    id: "presentation-production",
    kind: "workflow",
    effect: "derive",
    labelKey: "spaces.creation.presentation.create",
    icon: "file-output",
    decide: decideKnowledgeIntent,
  },
  {
    id: "video-production",
    kind: "workflow",
    effect: "derive",
    labelKey: "spaces.creation.video.generation.create",
    icon: "sparkles",
    decide: decideVideoGeneration,
  },
  {
    id: "social-content-package",
    kind: "workflow",
    effect: "derive",
    labelKey: "spaces.creation.social.contentPackage",
    icon: "sparkles",
    decide: decideKnowledgeIntent,
  },
  {
    id: "remove-image-background",
    kind: "atomic",
    effect: "derive",
    labelKey: "spaces.creation.image.removeBackground",
    icon: "scan",
    decide: ({ canRemoveBackground }) => (canRemoveBackground ? AVAILABLE : HIDDEN),
  },
  {
    id: "subtitled-video-production",
    kind: "workflow",
    effect: "derive",
    labelKey: "spaces.creation.video.subtitles.production",
    icon: "captions",
    decide: ({ canExtractSubtitles }) => (canExtractSubtitles ? AVAILABLE : HIDDEN),
  },
  {
    id: "meeting-content-package",
    kind: "workflow",
    effect: "derive",
    labelKey: "spaces.creation.meeting.contentPackage",
    icon: "sparkles",
    decide: ({ canCreateMeetingPackage }) => (canCreateMeetingPackage ? AVAILABLE : HIDDEN),
  },
  {
    id: "render-subtitled-video",
    kind: "atomic",
    effect: "derive",
    labelKey: "spaces.creation.video.subtitles.create",
    icon: "captions",
    decide: ({ canBurnSubtitles }) => (canBurnSubtitles ? AVAILABLE : HIDDEN),
  },
  {
    id: "extract-subtitles",
    kind: "atomic",
    effect: "derive",
    labelKey: "spaces.creation.video.subtitles.createResource",
    icon: "captions",
    decide: ({ canExtractSubtitles }) => (canExtractSubtitles ? AVAILABLE : HIDDEN),
  },
] as const satisfies readonly CreationIntentDefinition[];

export type BuiltinCreationIntent = (typeof creationIntentDefinitions)[number];
export type CreationIntentId = BuiltinCreationIntent["id"];
export type CreationIntentDecisions = Record<CreationIntentId, CreationIntentDecision>;

export function decideCreationIntents(context: CreationSelectionContext): CreationIntentDecisions {
  return Object.fromEntries(
    creationIntentDefinitions.map((definition) => [definition.id, definition.decide(context)]),
  ) as CreationIntentDecisions;
}
