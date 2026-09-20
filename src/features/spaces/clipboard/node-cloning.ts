import { duplicateSpaceAsset } from "../assets/asset-operations";
import { duplicateTiptapAssets } from "../assets/managed-assets";
import { isSpaceManagedAssetPayload, isSpaceWebsitePayload } from "../model/nodes/capabilities";
import type { SpaceNodePayload } from "../model/nodes/node-payload";
import { validateSpaceNodePayload } from "../model/nodes/schema/node-payload";

export interface ContentCloneContext {
  identities: ReadonlyMap<string, string>;
  sourceSpaceId: string;
  targetSpaceId: string;
  assets: string[];
}
export async function cloneNodeData(source: SpaceNodePayload, context: ContentCloneContext) {
  const data = validateSpaceNodePayload(source);
  if (data.kind === "credential") data.credentialConfigured = false;
  if (data.kind === "markdown") delete data.derivationState;
  remapNavigation(data, context);
  if (isSpaceManagedAssetPayload(data)) {
    data.assetPath = await duplicateTracked(data.assetPath, context);
    if ((data.kind === "video" || data.kind === "audio") && data.origin) {
      if (data.previewPath) data.previewPath = await duplicateTracked(data.previewPath, context);
    } else {
      data.processingState = "processing";
      delete data.previewPath;
    }
  }
  if (isSpaceWebsitePayload(data)) {
    if (data.faviconPath) data.faviconPath = await duplicateTracked(data.faviconPath, context);
    if (data.previewImagePath) data.previewImagePath = await duplicateTracked(data.previewImagePath, context);
  }
  await duplicateTiptapAssets(data, (path) => duplicateTracked(path, context));
  return data;
}
function remapNavigation(data: SpaceNodePayload, context: ContentCloneContext) {
  if (
    data.kind === "portal" &&
    data.targetSpaceId === context.sourceSpaceId &&
    data.targetNodeId &&
    context.identities.has(data.targetNodeId)
  ) {
    if (data.targetSceneId && context.sourceSpaceId !== context.targetSpaceId)
      throw new Error(
        "A Portal to a Saved View must be copied with a native Space package when changing its target Space",
      );
    data.targetNodeId = context.identities.get(data.targetNodeId)!;
    data.targetSpaceId = context.targetSpaceId;
  }
  if (data.kind === "subtitle") {
    const video = context.identities.get(data.sourceVideoNodeId);
    if (video) data.sourceVideoNodeId = video;
    else if (context.sourceSpaceId !== context.targetSpaceId)
      throw new Error("Copy the source video together with subtitles when pasting into another Space");
  }
}
async function duplicateTracked(path: string, context: ContentCloneContext) {
  const copy = await duplicateSpaceAsset(path);
  context.assets.push(copy);
  return copy;
}
