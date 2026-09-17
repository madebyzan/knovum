import { duplicateSpaceAsset, removeSpaceAssets } from "../assets/asset-operations";
import { duplicateTiptapAssets } from "../assets/managed-assets";
import { createSpaceNode } from "../canvas/node";
import type { PositionedSpaceNode, SpaceCanvasNode } from "../model/nodes/canvas-node";
import { isSpaceManagedAssetPayload, isSpaceWebsitePayload } from "../model/nodes/capabilities";
import type { SpaceNodePayload } from "../model/nodes/node-payload";
import { validateSpaceNodePayload } from "../model/nodes/schema/node-payload";
import { createSpaceClipboardPayload } from "./payload";
import type { SpaceClipboardPayload } from "./payload-types";

export async function duplicateSpaceNodes(nodes: readonly PositionedSpaceNode[], offset = 24) {
  if (nodes.length === 0) return [];
  const payload = createSpaceClipboardPayload(nodes);
  const minX = Math.min(...payload.nodes.map((node) => node.position.x));
  const minY = Math.min(...payload.nodes.map((node) => node.position.y));
  return cloneClipboardNodes(payload, { x: minX + offset, y: minY + offset });
}

export async function cloneClipboardNodes(payload: SpaceClipboardPayload, anchor: { x: number; y: number }) {
  const ids = new Map(payload.nodes.map((node) => [node.id, crypto.randomUUID()]));
  const context: CloneContext = {
    ids,
    copiedSections: new Set(
      payload.nodes.filter((node) => node.data.kind === "section").map((node) => node.id),
    ),
    createdAssets: [],
  };
  const minX = Math.min(...payload.nodes.map((node) => node.position.x));
  const minY = Math.min(...payload.nodes.map((node) => node.position.y));
  try {
    const clones: SpaceCanvasNode[] = [];
    for (const node of payload.nodes) {
      const cloneId = ids.get(node.id)!;
      // oxlint-disable-next-line no-await-in-loop -- Asset copies are transactional and must be tracked before the next clone starts.
      const data = await cloneData(node.data, context);
      clones.push(
        createSpaceNode(
          data,
          { x: anchor.x + node.position.x - minX, y: anchor.y + node.position.y - minY },
          cloneId,
        ),
      );
    }
    return clones;
  } catch (error) {
    await removeSpaceAssets(context.createdAssets);
    throw error;
  }
}

interface CloneContext {
  ids: Map<string, string>;
  copiedSections: Set<string>;
  createdAssets: string[];
}

async function cloneData(source: SpaceNodePayload, context: CloneContext) {
  const data = validateSpaceNodePayload(source);
  if (data.kind === "credential") data.credentialConfigured = false;
  if (data.kind === "markdown" && data.derivationState) delete data.derivationState;
  remapSection(data, context);
  remapReferences(data, context);
  await duplicateManagedAsset(data, context);
  await duplicateWebsiteAssets(data, context);
  await duplicateTiptapAssets(data, (path) => duplicateTracked(path, context));
  return data;
}

async function duplicateManagedAsset(data: SpaceNodePayload, context: CloneContext) {
  if (!isSpaceManagedAssetPayload(data)) return;
  data.assetPath = await duplicateTracked(data.assetPath, context);
  if ((data.kind === "video" || data.kind === "audio") && data.origin) {
    if (data.previewPath) data.previewPath = await duplicateTracked(data.previewPath, context);
    return;
  }
  data.processingState = "processing";
  delete data.previewPath;
}

async function duplicateWebsiteAssets(data: SpaceNodePayload, context: CloneContext) {
  if (!isSpaceWebsitePayload(data)) return;
  if (data.faviconPath) data.faviconPath = await duplicateTracked(data.faviconPath, context);
  if (data.previewImagePath) data.previewImagePath = await duplicateTracked(data.previewImagePath, context);
}

function remapReferences(data: SpaceNodePayload, context: CloneContext) {
  if (data.kind === "portal" && data.targetNodeId && context.ids.has(data.targetNodeId)) {
    data.targetNodeId = context.ids.get(data.targetNodeId)!;
  } else if (data.kind === "subtitle" && context.ids.has(data.sourceVideoNodeId)) {
    data.sourceVideoNodeId = context.ids.get(data.sourceVideoNodeId)!;
  }
}

async function duplicateTracked(path: string, context: CloneContext) {
  const copy = await duplicateSpaceAsset(path);
  context.createdAssets.push(copy);
  return copy;
}

function remapSection(data: SpaceNodePayload, context: CloneContext) {
  if (!data.sectionId) return;
  if (!context.copiedSections.has(data.sectionId)) {
    delete data.sectionId;
    return;
  }
  data.sectionId = context.ids.get(data.sectionId)!;
}
