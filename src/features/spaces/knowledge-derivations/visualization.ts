import type { ImageAspectRatio } from "../contracts/generated/ImageAspectRatio";
export type { ImageAspectRatio } from "../contracts/generated/ImageAspectRatio";
import type { ExecutionTarget } from "../../../foundation/ai/schema/models";
import { i18n } from "../../../foundation/i18n/i18n";
import type { useSpaceStore } from "../session/space-store";
import type { GroundedDocumentSource } from "./catalog/grounded-document";
import { startDerivationAction } from "./start-action";

export interface VisualizationDraft {
  title: string;
  intent: string;
  style: string;
  aspectRatio: ImageAspectRatio;
  target?: ExecutionTarget | undefined;
}

const imageAspectRatios: ReadonlySet<unknown> = new Set(["1:1", "3:2", "2:3", "16:9", "9:16"]);

export function isImageAspectRatio(value: unknown): value is ImageAspectRatio {
  return imageAspectRatios.has(value);
}
interface GenerateVisualizationContext {
  store: ReturnType<typeof useSpaceStore>;
  sources: readonly GroundedDocumentSource[];
  draft: VisualizationDraft;
}

export function generateVisualization({ store, sources, draft }: GenerateVisualizationContext) {
  return startDerivationAction({
    store,
    sources,
    actionId: "generate-visualization",
    title: i18n.global.t("spaces.creation.image.visualization.task"),
    target: draft.target,
    parameters: {
      title: draft.title.trim(),
      intent: draft.intent.trim(),
      style: draft.style,
      aspectRatio: draft.aspectRatio,
    },
  });
}
