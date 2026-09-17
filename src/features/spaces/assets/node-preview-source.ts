import { convertFileSrc } from "@tauri-apps/api/core";
import { withQuery } from "ufo";

import type { SpaceNodePayload } from "../model/nodes/node-payload";

export function spaceNodePreviewSource(data: SpaceNodePayload) {
  if ("processingState" in data && data.processingState === "importing") return "";
  if (data.kind === "image") return spaceAssetSource(data.assetPath, data.sourceRevision);
  if (data.kind === "video" || data.kind === "file") {
    return data.previewPath ? spaceAssetSource(data.previewPath, data.sourceRevision) : "";
  }
  if (data.kind === "website" && data.previewImagePath) return spaceAssetSource(data.previewImagePath);
  return "";
}

export function spaceCanvasOverviewSource(data: SpaceNodePayload) {
  if (data.kind === "image" && data.previewPath) {
    return spaceAssetSource(data.previewPath, data.sourceRevision);
  }
  return spaceNodePreviewSource(data);
}

export function spaceAssetSource(path: string, revision?: string) {
  const source = convertFileSrc(path);
  return revision ? withQuery(source, { v: revision }) : source;
}
