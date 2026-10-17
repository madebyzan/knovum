import type {
  CreationIntentDecision,
  CreationIntentDefinition,
  CreationSelectionContext,
} from "./creation-intent";
import { MAX_CREATION_SOURCES } from "./creation-source";

const AVAILABLE: CreationIntentDecision = { state: "ready" };
const HIDDEN: CreationIntentDecision = { state: "hidden" };

/**
 * Creating from a selection depends on the selection, not on background work:
 * a resource the Run can send is enough. Readiness of its knowledge is the
 * Run's problem, and the Run says so while it prepares.
 */
function decideCreation(context: CreationSelectionContext): CreationIntentDecision {
  if (context.selectionCount === 0) return HIDDEN;
  if (context.selectionCount > MAX_CREATION_SOURCES) {
    return { state: "blocked", reason: "too-many-sources" };
  }
  return context.sourceCount > 0 ? AVAILABLE : { state: "blocked", reason: "no-usable-source" };
}

function decideVideoGeneration(context: CreationSelectionContext): CreationIntentDecision {
  const creation = decideCreation(context);
  if (creation.state !== "ready") return creation;
  return context.imageCount > 10 ? { state: "blocked", reason: "too-many-video-images" } : AVAILABLE;
}

export const CREATION_INTENT_DEFINITIONS = [
  { id: "draft-document", icon: "file-pen", decide: decideCreation },
  { id: "generate-visualization", icon: "image", decide: decideCreation },
  { id: "generate-video", icon: "sparkles", decide: decideVideoGeneration },
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

export type CreationIntentId = (typeof CREATION_INTENT_DEFINITIONS)[number]["id"];
export type CreationIntentDecisions = Record<CreationIntentId, CreationIntentDecision>;

export function decideCreationIntents(context: CreationSelectionContext): CreationIntentDecisions {
  return Object.fromEntries(
    CREATION_INTENT_DEFINITIONS.map((definition) => [definition.id, definition.decide(context)]),
  ) as CreationIntentDecisions;
}
