import type { SpaceCodePayload, SpaceFilePayload } from "../model/nodes/node-payload";
import { classifyFilePresentation, fileCaptionKey, type SpaceCaptionKey } from "./file-presentation";
import { formatByteSize } from "./format-byte-size";

/**
 * Media is deliberately absent: an image, a video or an audio player already says what it is, and
 * `docs/spaces.md` keeps their canvas treatment quiet. Only resources whose content cannot speak
 * for itself carries a caption strip.
 */
export type SpaceCaptionData = SpaceFilePayload | SpaceCodePayload;

export interface SpaceCaption {
  labelKey: `spaces.transfer.fileType.${SpaceCaptionKey}`;
  detail?: string;
}

export function spaceNodeCaption(data: SpaceCaptionData): SpaceCaption {
  const labelKey: SpaceCaption["labelKey"] = `spaces.transfer.fileType.${captionKey(data)}`;
  const detail = formatByteSize(data.size);
  return detail === undefined ? { labelKey } : { labelKey, detail };
}

function captionKey(data: SpaceCaptionData): SpaceCaptionKey {
  if (data.kind === "file") {
    return fileCaptionKey(classifyFilePresentation(data.fileName ?? data.title));
  }
  return data.kind;
}
