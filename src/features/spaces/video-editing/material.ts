import type { SpaceCanvasNode } from "../model/nodes/canvas-node";
import type { SpaceAudioPayload, SpaceImagePayload, SpaceVideoPayload } from "../model/nodes/node-payload";
import { videoAssetSchema } from "../model/nodes/schema/video-edit-media";
import type { VideoAsset, VideoEditProject } from "./project";

type ReadyVideoMaterial = (SpaceVideoPayload | SpaceImagePayload | SpaceAudioPayload) & {
  sourceRevision: string;
};
export type VideoMaterialNode = SpaceCanvasNode<ReadyVideoMaterial>;
export function isVideoMaterial(node: SpaceCanvasNode): node is VideoMaterialNode {
  return (
    ["video", "image", "audio"].includes(node.data.kind) &&
    "assetPath" in node.data &&
    !!node.data.sourceRevision &&
    !("processingState" in node.data && node.data.processingState)
  );
}
export function materialMatches(
  asset: VideoAsset,
  node: SpaceCanvasNode | undefined,
): node is VideoMaterialNode {
  return (
    !!node &&
    isVideoMaterial(node) &&
    node.id === asset.nodeId &&
    node.data.kind === asset.kind &&
    node.data.sourceRevision === asset.sourceRevision
  );
}
export function measuredVideoAsset(
  node: VideoMaterialNode,
  measurements: { durationMs: number; width: number; height: number },
): VideoAsset {
  return videoAssetSchema.parse({
    nodeId: node.id,
    sourceRevision: node.data.sourceRevision,
    kind: node.data.kind,
    ...measurements,
  });
}

export function keepReferencedVideoAssets(project: VideoEditProject): VideoEditProject {
  const references = new Set([
    project.assets[0]!.nodeId,
    ...project.videoTrack.clips.map(({ assetId }) => assetId),
    ...project.audioTrack.clips.map(({ assetId }) => assetId),
  ]);
  return { ...project, assets: project.assets.filter(({ nodeId }) => references.has(nodeId)) };
}
