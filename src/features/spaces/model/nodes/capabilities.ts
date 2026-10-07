import type {
  SpaceAudioPayload,
  SpaceCodePayload,
  SpaceFilePayload,
  SpaceImagePayload,
  SpaceMarkdownPayload,
  SpaceNodePayload,
  SpaceNotePayload,
  SpaceTextPayload,
  SpaceVideoPayload,
  SpaceWebsitePayload,
} from "./node-payload";

export type SpaceManagedAssetPayload =
  | SpaceImagePayload
  | SpaceVideoPayload
  | SpaceAudioPayload
  | SpaceFilePayload
  | SpaceCodePayload;

export type SpaceMediaPayload = SpaceAudioPayload | SpaceVideoPayload;
export type SpaceEditorPayload = SpaceNotePayload | SpaceMarkdownPayload;
export type SpaceCanvasTextPayload = SpaceNotePayload | SpaceTextPayload;

export function isSpaceManagedAssetPayload(data: SpaceNodePayload): data is SpaceManagedAssetPayload {
  return ["image", "video", "audio", "file", "code"].includes(data.kind);
}

export function isSpaceMediaPayload(data: SpaceNodePayload): data is SpaceMediaPayload {
  return data.kind === "audio" || data.kind === "video";
}

export function isSpaceWebsitePayload(data: SpaceNodePayload): data is SpaceWebsitePayload {
  return data.kind === "website";
}
