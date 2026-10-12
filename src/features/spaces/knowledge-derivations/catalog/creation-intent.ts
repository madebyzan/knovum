import type { ActionAvailability } from "../../../../shared/contracts/action-availability";

export type CreationIntentIcon = "captions" | "file-output" | "file-pen" | "image" | "scan" | "sparkles";
export type DerivationUnavailableReason =
  | "needs-indexed-knowledge"
  | "needs-document-recognition"
  | "too-many-video-images"
  | "active-node-operation";

export type CreationIntentDecision = ActionAvailability<DerivationUnavailableReason>;
export type DocumentFormat = "docx" | "pdf" | "txt" | "html" | "pptx";

export interface CreationSelectionContext {
  readonly selectionCount: number;
  readonly hasKnowledgeSources: boolean;
  readonly hasVideoSources: boolean;
  readonly imageCount: number;
  readonly canRemoveBackground: boolean;
  readonly canAddVideoSubtitles: boolean;
  readonly documentTargets: readonly DocumentFormat[];
  readonly documentRecognitionRequired: boolean;
}

export interface CreationIntentDefinition {
  readonly id: string;
  readonly labelKey: string;
  readonly icon: CreationIntentIcon;
  readonly documentFormat?: DocumentFormat;
  readonly decide: (context: CreationSelectionContext) => CreationIntentDecision;
}
