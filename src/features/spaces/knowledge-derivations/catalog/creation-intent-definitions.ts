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
  const knowledge = context.hasVideoSources ? AVAILABLE : decideKnowledgeIntent(context);
  if (knowledge.state !== "ready") return knowledge;
  return context.imageCount > 10 ? { state: "blocked", reason: "too-many-video-images" } : AVAILABLE;
}

export const creationIntentDefinitions = [
  {
    id: "draft-document",
    labelKey: "spaces.creation.document.generate",
    icon: "file-pen",
    decide: decideKnowledgeIntent,
  },
  {
    id: "generate-visualization",
    labelKey: "spaces.creation.image.visualize",
    icon: "image",
    decide: decideKnowledgeIntent,
  },
  {
    id: "export-document-as-docx",
    labelKey: "spaces.creation.document.export.docx",
    icon: "file-output",
    decide: ({ canExportDocument }) => (canExportDocument ? AVAILABLE : HIDDEN),
  },
  {
    id: "export-document-as-pdf",
    labelKey: "spaces.creation.document.export.pdf",
    icon: "file-output",
    decide: ({ canExportDocument }) => (canExportDocument ? AVAILABLE : HIDDEN),
  },
  {
    id: "export-document-as-txt",
    labelKey: "spaces.creation.document.export.txt",
    icon: "file-output",
    decide: ({ canExportDocument }) => (canExportDocument ? AVAILABLE : HIDDEN),
  },
  {
    id: "render-presentation",
    labelKey: "spaces.creation.presentation.create",
    icon: "file-output",
    decide: ({ canExportDocument }) => (canExportDocument ? AVAILABLE : HIDDEN),
  },
  {
    id: "generate-video",
    labelKey: "spaces.creation.video.generation.create",
    icon: "sparkles",
    decide: decideVideoGeneration,
  },
  {
    id: "remove-image-background",
    labelKey: "spaces.creation.image.removeBackground",
    icon: "scan",
    decide: ({ canRemoveBackground }) => (canRemoveBackground ? AVAILABLE : HIDDEN),
  },
  {
    id: "add-video-subtitles",
    labelKey: "spaces.creation.video.subtitles.add",
    icon: "captions",
    decide: ({ canAddVideoSubtitles }) => (canAddVideoSubtitles ? AVAILABLE : HIDDEN),
  },
] as const satisfies readonly CreationIntentDefinition[];

type CreationIntent = (typeof creationIntentDefinitions)[number];
export type CreationIntentId = CreationIntent["id"];
export type CreationIntentDecisions = Record<CreationIntentId, CreationIntentDecision>;

export function decideCreationIntents(context: CreationSelectionContext): CreationIntentDecisions {
  return Object.fromEntries(
    creationIntentDefinitions.map((definition) => [definition.id, definition.decide(context)]),
  ) as CreationIntentDecisions;
}
