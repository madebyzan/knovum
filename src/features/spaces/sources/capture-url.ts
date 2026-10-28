import { info as logInfo, warn as logWarn } from "@tauri-apps/plugin-log";

import { decodeAppError } from "../../../foundation/errors/app-error";
import { formatErrorDiagnostic } from "../../../foundation/errors/error-format";
import type { PendingContentContext } from "../assets/import-contract";
import { acquireMediaSource } from "../media/import/acquisition";
import { allocateMediaImportAsset } from "../media/import/ipc";
import type { SpaceCanvasNode } from "../model/nodes/canvas-node";
import type { SpaceMediaPayload } from "../model/nodes/capabilities";
import type { SpaceNodePayload } from "../model/nodes/node-payload";
import type { NodePayloadChangeInput } from "../nodes/node-operations";
import { addSavedWebsite } from "../website/capture";
import { normalizeSpaceUrl } from "../website/url";
import { classifyUrlSource } from "./url-source-kind";

export interface UrlCaptureContext extends PendingContentContext {
  getActiveSpaceId(): string | undefined;
  nodes: SpaceCanvasNode[];
  addContent(data: SpaceNodePayload, position?: { x: number; y: number }): Promise<SpaceCanvasNode[]>;
  saveNodeContentForSpace(spaceId: string, change: NodePayloadChangeInput): Promise<unknown>;
}

/**
 * Captures what a link points at.
 *
 * A page is saved as a website. A media link says which resource it will become, so the node for
 * it is reserved before the first byte: the arriving media is ordinary canvas content from the
 * moment the link is dropped — it can be moved, selected and left alone while it finishes — and
 * the Task Center carries the progress of the work that belongs to it.
 */
export async function captureSpaceUrl(
  context: UrlCaptureContext,
  url: string,
  position?: { x: number; y: number },
) {
  const spaceId = context.getActiveSpaceId();
  if (!spaceId) throw new Error("A Space must be active");
  const normalized = normalizeSpaceUrl(url);
  if (!normalized) throw new Error("The URL must be HTTP or HTTPS");
  const target = position ?? { x: 0, y: 0 };
  const sourceKind = classifyUrlSource(new URL(normalized));
  void logInfo(`[spaces.url-capture] route=${sourceKind}`);
  if (sourceKind === "website") {
    return addSavedWebsite({ ...context, activeSpaceId: spaceId }, normalized, target);
  }
  return importMediaSource(context, { spaceId, url: normalized, kind: sourceKind, target });
}

interface MediaSourceImportInput {
  spaceId: string;
  url: string;
  kind: "audio" | "video";
  target: { x: number; y: number };
}

async function importMediaSource(context: UrlCaptureContext, input: MediaSourceImportInput) {
  const jobId = crypto.randomUUID();
  const assetPath = await allocateMediaImportAsset(jobId, input.kind);
  const [node] = await context.createPendingContentForSpace(
    input.spaceId,
    arrivingMedia(input, assetPath),
    input.target,
  );
  if (!node) throw new Error("The arriving media could not be placed on the canvas");
  try {
    const acquisition = await acquireMediaSource(input.url, {
      jobId,
      nodeId: node.id,
      kind: input.kind,
      title: node.data.title,
    });
    if (acquisition.outcome !== "imported") throw new Error(`Media source is unsupported: ${input.url}`);
    return await settle(context, input.spaceId, node, acquisition.payload);
  } catch (error) {
    // Whatever was reserved for this media never arrived, so it is taken away with its asset. The
    // failure itself belongs to whoever asked for the capture.
    await discard(context, input.spaceId, node);
    if (decodeAppError(error)?.code === "media_acquisition_cancelled") return undefined;
    void logWarn(`Media import failed for ${input.url}: ${formatErrorDiagnostic(error)}`);
    throw error;
  }
}

/** The content of a media node whose bytes are still arriving. */
function arrivingMedia(input: MediaSourceImportInput, assetPath: string): SpaceMediaPayload {
  const common = {
    title: arrivingTitle(input.url),
    assetPath,
    fileName: assetPath.split(/[/\\]/u).at(-1) ?? input.kind,
    processingState: "importing",
  } as const;
  return input.kind === "audio" ? { kind: "audio", ...common } : { kind: "video", ...common };
}

/** What to call media that has not introduced itself yet: the link it is arriving from. */
function arrivingTitle(url: string) {
  const { host, pathname, search } = new URL(url);
  return `${host}${pathname}${search}`.slice(0, 512);
}

async function settle(
  context: UrlCaptureContext,
  spaceId: string,
  node: SpaceCanvasNode,
  payload: SpaceMediaPayload,
) {
  // The import publishes into the asset the node was reserved with, as the kind the link asked
  // for, so the node the canvas has been showing is the node that settles.
  if ("assetPath" in node.data && node.data.assetPath !== payload.assetPath)
    throw new Error("The imported media was written somewhere other than its reserved asset");
  if (node.data.kind !== payload.kind)
    throw new Error("The imported media is not the kind its link asked for");
  await context.savePendingNodeContentForSpace(spaceId, {
    id: node.id,
    expectedContent: node.data,
    data: payload,
  });
  node.data = payload;
  context.finalizePendingContent(spaceId, [node], { reveal: context.getActiveSpaceId() === spaceId });
  void logInfo("[spaces.url-capture] media node settled");
  return node;
}

/**
 * Takes the arriving media away. Its reserved asset goes with the node: discarding a node reclaims
 * the assets only it referenced, so removing the asset here as well would race that reclamation
 * while the node still holds it.
 */
async function discard(context: UrlCaptureContext, spaceId: string, node: SpaceCanvasNode) {
  try {
    await context.discardPendingContent(spaceId, node.id);
  } catch (error) {
    void logWarn(
      `Arriving media could not be removed after a failed import: ${formatErrorDiagnostic(error)}`,
    );
  }
}
