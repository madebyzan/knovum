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

function decideDocumentConversion(
  context: CreationSelectionContext,
  format: NonNullable<CreationIntentDefinition["documentFormat"]>,
): CreationIntentDecision {
  if (!context.documentTargets.includes(format)) return HIDDEN;
  return context.documentRecognitionRequired
    ? { state: "blocked", reason: "needs-document-recognition" }
    : AVAILABLE;
}

export const CREATION_INTENT_DEFINITIONS = [
  {
    id: "draft-document",
    icon: "file-pen",
    decide: decideKnowledgeIntent,
  },
  {
    id: "generate-visualization",
    icon: "image",
    decide: decideKnowledgeIntent,
  },
  {
    id: "convert-document-to-docx",
    documentFormat: "docx",
    icon: "file-output",
    decide: (context) => decideDocumentConversion(context, "docx"),
  },
  {
    id: "convert-document-to-pdf",
    documentFormat: "pdf",
    icon: "file-output",
    decide: (context) => decideDocumentConversion(context, "pdf"),
  },
  {
    id: "convert-document-to-txt",
    documentFormat: "txt",
    icon: "file-output",
    decide: (context) => decideDocumentConversion(context, "txt"),
  },
  {
    id: "convert-document-to-html",
    documentFormat: "html",
    icon: "file-output",
    decide: (context) => decideDocumentConversion(context, "html"),
  },
  {
    id: "render-presentation",
    documentFormat: "pptx",
    icon: "file-output",
    decide: (context) => decideDocumentConversion(context, "pptx"),
  },
  {
    id: "generate-video",
    icon: "sparkles",
    decide: decideVideoGeneration,
  },
  {
    id: "remove-image-background",
    icon: "scan",
    decide: ({ canRemoveBackground }) => (canRemoveBackground ? AVAILABLE : HIDDEN),
  },
  {
    id: "split-image-layers",
    icon: "layers",
    decide: ({ canSplitImageLayers }) => (canSplitImageLayers ? AVAILABLE : HIDDEN),
  },
  {
    id: "add-video-subtitles",
    icon: "captions",
    decide: ({ canAddVideoSubtitles }) => (canAddVideoSubtitles ? AVAILABLE : HIDDEN),
  },
] as const satisfies readonly CreationIntentDefinition[];

type CreationIntent = (typeof CREATION_INTENT_DEFINITIONS)[number];
export type CreationIntentId = CreationIntent["id"];
export type CreationIntentDecisions = Record<CreationIntentId, CreationIntentDecision>;

export const DOCUMENT_CONVERSION_DEFINITIONS = CREATION_INTENT_DEFINITIONS.filter(
  (definition) => "documentFormat" in definition,
);
export const DOCUMENT_CONVERSION_ACTION_IDS = DOCUMENT_CONVERSION_DEFINITIONS.map(({ id }) => id);

export function decideCreationIntents(context: CreationSelectionContext): CreationIntentDecisions {
  return Object.fromEntries(
    CREATION_INTENT_DEFINITIONS.map((definition) => [definition.id, definition.decide(context)]),
  ) as CreationIntentDecisions;
}
