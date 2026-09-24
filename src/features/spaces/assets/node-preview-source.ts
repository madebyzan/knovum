import { convertFileSrc } from "@tauri-apps/api/core";
import { withQuery } from "ufo";

import type { SpaceNodePayload } from "../model/nodes/node-payload";

export function spaceNodePreviewSource(data: SpaceNodePayload) {
  switch (data.kind) {
    case "image":
      return publishedPreview(data.assetPath, data.sourceRevision);
    case "video":
      return publishedPreview(data.previewPath, data.sourceRevision);
    case "file":
      return data.processingState === "importing"
        ? ""
        : optionalPreview(data.previewPath, data.sourceRevision);
    case "website":
      return optionalPreview(data.previewImagePath);
    default:
      return "";
  }
}

function publishedPreview(path: string | undefined, revision: string | undefined) {
  return path && revision ? spaceAssetSource(path, revision) : "";
}

function optionalPreview(path: string | undefined, revision?: string) {
  return path ? spaceAssetSource(path, revision) : "";
}

export function spaceCanvasOverviewSource(data: SpaceNodePayload) {
  if (data.kind === "image" && data.previewPath && data.sourceRevision) {
    return spaceAssetSource(data.previewPath, data.sourceRevision);
  }
  return spaceNodePreviewSource(data);
}

export function spaceAssetSource(path: string, revision?: string) {
  const source = convertFileSrc(path);
  return revision ? withQuery(source, { v: revision }) : source;
}
