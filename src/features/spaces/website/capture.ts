import { warn as logWarn } from "@tauri-apps/plugin-log";
import { compact } from "es-toolkit";
import pLimit from "p-limit";

import { removeSpaceAssets } from "../assets/asset-operations";
import type { SpaceCanvasNode } from "../model/nodes/canvas-node";
import type { SpaceNodePayload, SpaceWebsitePayload } from "../model/nodes/node-payload";
import { websiteDefaultTitle } from "./url";
import { cacheWebsiteAsset, fetchWebsiteResource } from "./website-resource";

const websiteCaptureConcurrency = pLimit(3);

interface WebsiteCaptureContext {
  activeSpaceId?: string;
  nodes: SpaceCanvasNode[];
  addContent: (data: SpaceNodePayload, position?: { x: number; y: number }) => Promise<SpaceCanvasNode[]>;
  saveNodeDataForSpace(spaceId: string, id: string, data: SpaceNodePayload): Promise<void>;
}

interface WebsiteBinding {
  context: WebsiteCaptureContext;
  spaceId: string;
  id: string;
  data: SpaceWebsitePayload;
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
      logWarn(`Website metadata state could not be saved for ${node.id}: ${String(error)}`),
    );
  }
  return node;
}

export async function refreshSavedWebsite(context: WebsiteCaptureContext, id: string, url: string) {
  const spaceId = requireSpace(context);
  const node = context.nodes.find((candidate) => candidate.id === id);
  if (!node || node.data.kind !== "website") return;
  const next = { ...node.data, metadataState: "loading" } as SpaceWebsitePayload;
  await context.saveNodeDataForSpace(spaceId, id, next);
  await websiteCaptureConcurrency(() =>
    hydrateSavedWebsite({ context, spaceId, id, data: next }, url, { preserveTitle: true }),
  );
}

async function hydrateSavedWebsite(
  target: WebsiteBinding,
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
    await persistWebsite(target, paths, {
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
    void logWarn(`Website metadata hydration failed for ${target.id} (${url}): ${String(error)}`);
    await markWebsiteFailed(target);
  }
}

async function hydrateDirectImage(
  target: WebsiteBinding,
  url: string,
  previewImagePath: string,
  behavior: { preserveTitle: boolean },
) {
  try {
    await persistWebsite(
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
    void logWarn(`Direct website image hydration failed for ${target.id} (${url}): ${String(error)}`);
    await markWebsiteFailed(target);
  }
}

async function markWebsiteFailed(target: WebsiteBinding): Promise<void> {
  await saveBinding(target, { ...target.data, metadataState: "failed" });
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
    void logWarn(`Website ${kind} could not be cached: ${String(error)}`);
    return undefined;
  }
}

async function persistWebsite(
  target: WebsiteBinding,
  paths: readonly (string | undefined)[],
  metadata: Partial<Omit<SpaceWebsitePayload, "kind">>,
  explicitPreviewPath?: string,
) {
  const cachedPaths = compact(paths);
  const [faviconPath, previewImagePath] = paths;
  const nextPreviewPath = explicitPreviewPath ?? previewImagePath;
  const previous: SpaceWebsitePayload = { ...target.data };
  delete previous.siteName;
  delete previous.description;
  delete previous.contentPreview;
  delete previous.websiteSections;
  delete previous.canonicalUrl;
  delete previous.faviconPath;
  delete previous.previewImagePath;
  const next: SpaceWebsitePayload = { ...previous, ...metadata };
  if (faviconPath && !explicitPreviewPath) next.faviconPath = faviconPath;
  if (nextPreviewPath) next.previewImagePath = nextPreviewPath;
  const stalePaths = compact([target.data.faviconPath, target.data.previewImagePath]).filter(
    (path) => path !== next.faviconPath && path !== next.previewImagePath,
  );
  try {
    await saveBinding(target, next);
    await removeSpaceAssets(stalePaths);
  } catch (error) {
    await removeSpaceAssets(cachedPaths);
    throw error;
  }
}

async function saveBinding(target: WebsiteBinding, data: SpaceWebsitePayload) {
  await target.context.saveNodeDataForSpace(target.spaceId, target.id, data);
  target.data = data;
}

function binding(context: WebsiteCaptureContext, spaceId: string, node: SpaceCanvasNode): WebsiteBinding {
  if (node.data.kind !== "website") throw new Error("Website capture created a non-website node");
  return { context, spaceId, id: node.id, data: node.data };
}

function requireSpace(context: WebsiteCaptureContext) {
  if (!context.activeSpaceId) throw new Error("A Space must be active");
  return context.activeSpaceId;
}
