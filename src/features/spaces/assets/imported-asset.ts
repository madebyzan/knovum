import type {
  SpaceAudioPayload,
  SpaceCodePayload,
  SpaceFilePayload,
  SpaceImagePayload,
  SpaceMarkdownPayload,
  SpaceArchiveMember,
  SpaceVideoPayload,
} from "../model/nodes/node-payload";

type SpaceImportableData =
  | SpaceAudioPayload
  | SpaceCodePayload
  | SpaceFilePayload
  | SpaceImagePayload
  | SpaceMarkdownPayload
  | SpaceVideoPayload;

type SpaceAssetKind = SpaceImportableData["kind"];

export interface ImportedSpaceAsset {
  name: string;
  path?: string;
  kind: SpaceAssetKind;
  text?: string;
  previewText?: string;
  previewLanguage?: string;
  previewPath?: string;
  extractedText?: string;
  size?: number;
  modifiedAt?: number;
  sourceRevision?: string;
  processingState?: "importing" | "processing" | "failed";
  intrinsicWidth?: number;
  intrinsicHeight?: number;
  entries?: SpaceArchiveMember[];
}
