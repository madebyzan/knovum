import { omit } from "es-toolkit";
import type { Ref } from "vue";

import type { TranscriptSegment } from "../../../shared/contracts/generated/TranscriptSegment";

export interface SpaceMediaAnalysisPreview {
  language: string;
  progress: number;
  segments: TranscriptSegment[];
}

export type SpaceMediaAnalysisPreviews = Record<string, SpaceMediaAnalysisPreview>;

export interface AppendMediaAnalysisPreviewInput {
  previews: Ref<SpaceMediaAnalysisPreviews>;
  nodeId: string;
  progress: number;
  partial: Pick<SpaceMediaAnalysisPreview, "language" | "segments">;
}

export function appendSpaceMediaAnalysisPreview(input: AppendMediaAnalysisPreviewInput) {
  const { previews, nodeId, progress, partial } = input;
  const previous = previews.value[nodeId];
  previews.value = {
    ...previews.value,
    [nodeId]: {
      language: partial.language || previous?.language || "unknown",
      progress,
      segments: [...(previous?.segments ?? []), ...partial.segments],
    },
  };
}

export function clearSpaceMediaAnalysisPreview(previews: Ref<SpaceMediaAnalysisPreviews>, nodeId: string) {
  if (nodeId in previews.value) {
    previews.value = omit(previews.value, [nodeId]);
  }
}
