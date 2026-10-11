import { z } from "zod";

import { i18n } from "../../../foundation/i18n/instance";
import type { TaskTargetRef } from "../../../shared/contracts/generated/TaskTargetRef";
import type { ImageAspectRatio } from "../contracts/generated/ImageAspectRatio";
import type { GroundedDocumentSource } from "./catalog/grounded-document";
import { startDerivationAction } from "./start-action";

export interface VisualizationDraft {
  title: string;
  intent: string;
  style: string;
  aspectRatio: ImageAspectRatio;
  target?: TaskTargetRef | undefined;
}

export const imageAspectRatioSchema = z.enum([
  "1:1",
  "3:2",
  "2:3",
  "16:9",
  "9:16",
]) satisfies z.ZodType<ImageAspectRatio>;
interface GenerateVisualizationContext {
  sources: readonly GroundedDocumentSource[];
  draft: VisualizationDraft;
}

export function generateVisualization({ sources, draft }: GenerateVisualizationContext) {
  return startDerivationAction({
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
