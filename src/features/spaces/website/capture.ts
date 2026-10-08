import { warn as logWarn } from "@tauri-apps/plugin-log";
import pLimit from "p-limit";

import { formatErrorDiagnostic } from "../../../foundation/errors/error-format";
import type { SpaceCanvasNode } from "../model/nodes/canvas-node";
import type { SpaceNodePayload, SpaceWebsitePayload } from "../model/nodes/node-payload";
import type { NodePayloadChangeInput } from "../nodes/node-operations";
import {
  markSavedWebsiteFailedAfter,
  persistSavedWebsite,
  type SavedWebsiteBinding,
} from "./saved-website-persistence";
import { websiteDefaultTitle } from "./url";
import { cacheWebsiteAsset, fetchWebsiteResource } from "./website-resource";

const websiteCaptureConcurrency = pLimit(3);

interface WebsiteCaptureContext {
  activeSpaceId?: string;
  nodes: SpaceCanvasNode[];
  addContent: (data: SpaceNodePayload, position?: { x: number; y: number }) => Promise<SpaceCanvasNode[]>;
  saveNodeContentForSpace(spaceId: string, change: NodePayloadChangeInput): Promise<void>;
}

export async function addSavedWebsite(
  context: WebsiteCaptureContext,
  url: string,
  position?: { x: number; y: number },
) {
  const spaceId = requireSpace(context);
  const defaultTitle = websiteDefaultTitle(url);
  const [node] = await context.addContent(
    { kind: "website", title: defaultTitle, url, siteName: defaultTitle, metadataState: "loading" },
    position,
  );
  if (node?.data.kind === "website") {
    void websiteCaptureConcurrency(() =>
      hydrateSavedWebsite(binding(context, spaceId, node), url, { preserveTitle: false }),
    ).catch((error: unknown) =>
      logWarn(`Website metadata state could not be saved for ${node.id}: ${formatErrorDiagnostic(error)}`),
    );
  }
  return node;
}

export async function refreshSavedWebsite(context: WebsiteCaptureContext, id: string, url: string) {
  const spaceId = requireSpace(context);
  const node = context.nodes.find((candidate) => candidate.id === id);
  if (!node || node.data.kind !== "website") return;
  const next: SpaceWebsitePayload = { ...node.data, metadataState: "loading" };
  await context.saveNodeContentForSpace(spaceId, { id, expectedData: node.data, data: next });
  await websiteCaptureConcurrency(() =>
    hydrateSavedWebsite({ context, spaceId, id, data: next }, url, { preserveTitle: true }),
  );
}

async function hydrateSavedWebsite(
  target: SavedWebsiteBinding,
  url: string,
  behavior: { preserveTitle: boolean },
) {
  try {
    const resource = await fetchWebsiteResource(url);
    if (resource.kind === "image") {
      await hydrateDirectImage(target, resource.finalUrl, resource.assetPath, behavior);
      return;
    }
    const metadata = resource.metadata;
    const paths = await cacheMetadataAssets(metadata.faviconUrl, metadata.previewImageUrl);
    await persistSavedWebsite(target, paths, {
      ...(behavior.preserveTitle ? {} : { title: metadata.title }),
      pageTitle: metadata.title,
      url: metadata.finalUrl,
      metadataState: "ready",
      ...(metadata.siteName ? { siteName: metadata.siteName } : {}),
      ...(metadata.description ? { description: metadata.description } : {}),
      ...(metadata.contentPreview ? { contentPreview: metadata.contentPreview } : {}),
      ...(metadata.knowledgeSections ? { websiteSections: metadata.knowledgeSections } : {}),
      ...(metadata.canonicalUrl ? { canonicalUrl: metadata.canonicalUrl } : {}),
    });
  } catch (error) {
    void logWarn(
      `Website metadata hydration failed for ${target.id} (${url}): ${formatErrorDiagnostic(error)}`,
    );
    await markSavedWebsiteFailedAfter(target, error);
  }
}

async function hydrateDirectImage(
  target: SavedWebsiteBinding,
  url: string,
  previewImagePath: string,
  behavior: { preserveTitle: boolean },
) {
  try {
    await persistSavedWebsite(
      target,
      [undefined, previewImagePath],
      {
        ...(behavior.preserveTitle ? {} : { title: websiteDefaultTitle(url) }),
        url,
        siteName: websiteDefaultTitle(url),
        metadataState: "ready",
      },
      previewImagePath,
    );
  } catch (error) {
    void logWarn(
      `Direct website image hydration failed for ${target.id} (${url}): ${formatErrorDiagnostic(error)}`,
    );
    await markSavedWebsiteFailedAfter(target, error);
  }
}

async function cacheMetadataAssets(faviconUrl?: string, previewImageUrl?: string) {
  const [faviconPath, previewImagePath] = await Promise.all([
    cacheOptionalWebsiteAsset(faviconUrl, "favicon"),
    cacheOptionalWebsiteAsset(previewImageUrl, "preview image"),
  ]);
  return [faviconPath, previewImagePath] as const;
}

async function cacheOptionalWebsiteAsset(url: string | undefined, kind: string) {
  if (!url) return undefined;
  try {
    return await cacheWebsiteAsset(url);
  } catch (error) {
    void logWarn(`Website ${kind} could not be cached: ${formatErrorDiagnostic(error)}`);
    return undefined;
  }
}

function binding(
  context: WebsiteCaptureContext,
  spaceId: string,
  node: SpaceCanvasNode,
): SavedWebsiteBinding {
  if (node.data.kind !== "website") throw new Error("Website capture created a non-website node");
  return { context, spaceId, id: node.id, data: node.data };
}

function requireSpace(context: WebsiteCaptureContext) {
  if (!context.activeSpaceId) throw new Error("A Space must be active");
  return context.activeSpaceId;
}
