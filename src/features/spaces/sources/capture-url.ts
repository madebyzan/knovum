import { info as logInfo, warn as logWarn } from "@tauri-apps/plugin-log";

import { decodeAppError } from "../../../foundation/errors/app-error";
import { formatErrorDiagnostic } from "../../../foundation/errors/error-format";
import { removeSpaceAssets } from "../assets/asset-operations";
import { acquireMediaSource } from "../media/import/acquisition";
import type { ResolvedMedia } from "../media/import/schema/import";
import type { SpaceCanvasNode } from "../model/nodes/canvas-node";
import type { SpaceMediaPayload } from "../model/nodes/capabilities";
import type { SpaceNodePayload } from "../model/nodes/node-payload";
import type { NodePayloadChangeInput } from "../nodes/node-operations";
import { addSavedWebsite } from "../website/capture";
import { normalizeSpaceUrl } from "../website/url";
import { classifyUrlSource } from "./url-source-kind";

export interface UrlCaptureContext {
  getActiveSpaceId(): string | undefined;
  nodes: SpaceCanvasNode[];
  createPendingContentForSpace(
    spaceId: string,
    data: SpaceNodePayload,
    position?: { x: number; y: number },
  ): Promise<SpaceCanvasNode[]>;
  savePendingNodeContentForSpace(spaceId: string, change: NodePayloadChangeInput): Promise<void>;
  finalizePendingContent(spaceId: string, contentNodes: readonly SpaceCanvasNode[], reveal?: boolean): void;
  discardPendingContent(spaceId: string, nodeId: string): Promise<void>;
  addContent(data: SpaceNodePayload, position?: { x: number; y: number }): Promise<SpaceCanvasNode[]>;
  saveNodeContentForSpace(spaceId: string, change: NodePayloadChangeInput): Promise<unknown>;
}

/**
 * Captures what a link points at.
 *
 * A page is saved as a website. A media link is downloaded into a node that is reserved as soon as
 * the link says what it is, so the arriving media is ordinary canvas content from the first moment:
 * it can be moved, selected and left alone while it finishes, and the Task Center carries the
 * progress of the work that belongs to it.
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
  return importMediaSource(context, { spaceId, url: normalized, target });
}

interface MediaSourceImportInput {
  spaceId: string;
  url: string;
  target: { x: number; y: number };
}

async function importMediaSource(context: UrlCaptureContext, input: MediaSourceImportInput) {
  let reserved: SpaceCanvasNode | undefined;
  try {
    const acquisition = await acquireMediaSource(input.url, {
      resolved: async (media) => {
        const [node] = await context.createPendingContentForSpace(
          input.spaceId,
          arrivingMedia(media),
          input.target,
        );
        if (!node) throw new Error("The arriving media could not be placed on the canvas");
        reserved = node;
        return { nodeId: node.id, title: media.title };
      },
    });
    if (acquisition.outcome !== "imported") throw new Error(`Media source is unsupported: ${input.url}`);
    return await settle(context, input.spaceId, requireReserved(reserved), acquisition.payload);
  } catch (error) {
    // Whatever was reserved for this media never arrived, so it is taken away with its asset. The
    // failure itself belongs to whoever asked for the capture.
    await discard(context, input.spaceId, reserved);
    if (decodeAppError(error)?.code === "media_acquisition_cancelled") return undefined;
    void logWarn(`Media import failed for ${input.url}: ${formatErrorDiagnostic(error)}`);
    throw error;
  }
}

/** The content of a media node whose bytes are still arriving. */
function arrivingMedia(media: ResolvedMedia): SpaceMediaPayload {
  const common = {
    title: media.title,
    assetPath: media.assetPath,
    fileName: media.assetPath.split(/[/\\]/u).at(-1) ?? media.title,
    processingState: "importing",
    ...(media.durationSeconds === null ? {} : { durationSeconds: media.durationSeconds }),
  } as const;
  return media.kind === "audio" ? { kind: "audio", ...common } : { kind: "video", ...common };
}

function requireReserved(node: SpaceCanvasNode | undefined) {
  if (!node) throw new Error("The media source was never identified");
  return node;
}

async function settle(
  context: UrlCaptureContext,
  spaceId: string,
  node: SpaceCanvasNode,
  payload: SpaceMediaPayload,
) {
  // The engine publishes into the asset it named when the link resolved, which is the asset this
  // node was reserved with.
  if ("assetPath" in node.data && node.data.assetPath !== payload.assetPath)
    throw new Error("The imported media was written somewhere other than its reserved asset");
  await context.savePendingNodeContentForSpace(spaceId, {
    id: node.id,
    expectedContent: node.data,
    data: payload,
  });
  node.data = payload;
  context.finalizePendingContent(spaceId, [node], context.getActiveSpaceId() === spaceId);
  void logInfo("[spaces.url-capture] media node settled");
  return node;
}

async function discard(context: UrlCaptureContext, spaceId: string, node: SpaceCanvasNode | undefined) {
  if (!node) return;
  const assetPath = "assetPath" in node.data ? node.data.assetPath : undefined;
  const removals = await Promise.allSettled([
    context.discardPendingContent(spaceId, node.id),
    ...(assetPath ? [removeSpaceAssets([assetPath])] : []),
  ]);
  for (const removal of removals) {
    if (removal.status === "rejected")
      void logWarn(
        `Arriving media could not be removed after a failed import: ${formatErrorDiagnostic(removal.reason)}`,
      );
  }
}
