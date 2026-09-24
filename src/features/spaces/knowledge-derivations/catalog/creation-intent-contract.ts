import type { ActionAvailability } from "../../../../shared/contracts/action-availability";

export type CreationIntentKind = "atomic" | "workflow";
export type CreationIntentIcon = "captions" | "file-output" | "file-pen" | "image" | "scan" | "sparkles";
export type DerivationUnavailableReason =
  | "needs-indexed-knowledge"
  | "too-many-video-images"
  | "active-node-operation";

export type CreationIntentDecision = ActionAvailability<DerivationUnavailableReason>;

export interface CreationSelectionContext {
  readonly selectionCount: number;
  readonly hasKnowledgeSources: boolean;
  readonly imageCount: number;
  readonly canRemoveBackground: boolean;
  readonly canExtractSubtitles: boolean;
  readonly canBurnSubtitles: boolean;
  readonly canCreateMeetingPackage: boolean;
  readonly canExportDocument: boolean;
}

export interface CreationIntentDefinition {
  readonly id: string;
  readonly kind: CreationIntentKind;
  readonly effect: "derive" | "export";
  readonly labelKey: string;
  readonly icon: CreationIntentIcon;
  readonly decide: (context: CreationSelectionContext) => CreationIntentDecision;
}
