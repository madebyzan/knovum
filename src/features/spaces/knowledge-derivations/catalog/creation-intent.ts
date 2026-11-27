import type { ActionAvailability } from "../../../../shared/contracts/action-availability";

export type CreationIntentIcon =
  | "captions"
  | "clapperboard"
  | "file-pen"
  | "image"
  | "layers"
  | "scan"
  | "sticker";
type CreationIntentGroup = "create" | "utility";
export type DerivationUnavailableReason =
  | "no-usable-source"
  | "too-many-sources"
  | "too-many-video-images"
  | "active-node-operation";

export type CreationIntentDecision = ActionAvailability<DerivationUnavailableReason>;

export interface CreationSelectionContext {
  readonly selectionCount: number;
  /** Number of semantic knowledge nodes in the selection (excludes credentials, portals, plain text). */
  readonly semanticCount: number;
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
  readonly group: CreationIntentGroup;
  readonly icon: CreationIntentIcon;
  readonly decide: (context: CreationSelectionContext) => CreationIntentDecision;
}
