import type { ActionAvailability } from "../../../../shared/contracts/action-availability";

export type CreationIntentIcon =
  | "captions"
  | "file-pen"
  | "image"
  | "layers"
  | "scan"
  | "sparkles"
  | "sticker";
export type DerivationUnavailableReason =
  | "no-usable-source"
  | "too-many-sources"
  | "too-many-video-images"
  | "active-node-operation";

export type CreationIntentDecision = ActionAvailability<DerivationUnavailableReason>;

export interface CreationSelectionContext {
  readonly selectionCount: number;
  /** Resources that can be pinned and sent, whatever their index state. */
  readonly sourceCount: number;
  readonly imageCount: number;
  readonly canRemoveBackground: boolean;
  /** One published image is selected, so it can be read into a set of images. */
  readonly canDeriveImageSet: boolean;
  readonly canAddVideoSubtitles: boolean;
}

export interface CreationIntentDefinition {
  readonly id: string;
  readonly icon: CreationIntentIcon;
  readonly decide: (context: CreationSelectionContext) => CreationIntentDecision;
}
