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

export const CREATION_INTENT_DEFINITIONS = [
  {
    id: "draft-document",
    labelKey: "spaces.creation.document.generate",
    icon: "file-pen",
    decide: decideKnowledgeIntent,
  },
  {
    id: "generate-visualization",
    labelKey: "spaces.creation.image.visualization.title",
    icon: "image",
    decide: decideKnowledgeIntent,
  },
  {
    id: "convert-document-to-docx",
    documentFormat: "docx",
    labelKey: "spaces.creation.document.conversion.actions.docx",
    icon: "file-output",
    decide: ({ documentTargets }) => (documentTargets.includes("docx") ? AVAILABLE : HIDDEN),
  },
  {
    id: "convert-document-to-pdf",
    documentFormat: "pdf",
    labelKey: "spaces.creation.document.conversion.actions.pdf",
    icon: "file-output",
    decide: ({ documentTargets }) => (documentTargets.includes("pdf") ? AVAILABLE : HIDDEN),
  },
  {
    id: "convert-document-to-txt",
    documentFormat: "txt",
    labelKey: "spaces.creation.document.conversion.actions.txt",
    icon: "file-output",
    decide: ({ documentTargets }) => (documentTargets.includes("txt") ? AVAILABLE : HIDDEN),
  },
  {
    id: "convert-document-to-html",
    documentFormat: "html",
    labelKey: "spaces.creation.document.conversion.actions.html",
    icon: "file-output",
    decide: ({ documentTargets }) => (documentTargets.includes("html") ? AVAILABLE : HIDDEN),
  },
  {
    id: "render-presentation",
    documentFormat: "pptx",
    labelKey: "spaces.creation.presentation.create",
    icon: "file-output",
    decide: ({ documentTargets }) => (documentTargets.includes("pptx") ? AVAILABLE : HIDDEN),
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
