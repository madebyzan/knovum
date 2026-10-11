import { info as logInfo, warn as logWarn } from "@tauri-apps/plugin-log";
import { toast } from "vue-sonner";

import { decodeAppError } from "../../../foundation/errors/app-error";
import { formatErrorDiagnostic } from "../../../foundation/errors/error-format";
import { i18n } from "../../../foundation/i18n/instance";
import { removeSpaceAssets } from "../assets/asset-operations";
import { acquireMediaSource } from "../media/import/acquisition";
import type { MediaImportProgress } from "../media/import/schema/import";
import type { SpaceCanvasNode } from "../model/nodes/canvas-node";
import type { SpaceNodePayload } from "../model/nodes/node-payload";
import type { NodePayloadChangeInput } from "../nodes/node-operations";
import { addSavedWebsite } from "../website/capture";
import { normalizeSpaceUrl, websiteDefaultTitle } from "../website/url";
import type { SpaceUrlCaptureProjection } from "./url-capture-projection";
import { classifyUrlSource } from "./url-source-kind";

export interface UrlCaptureContext {
  getActiveSpaceId(): string | undefined;
  nodes: SpaceCanvasNode[];
  beginUrlCapture(input: Omit<SpaceUrlCaptureProjection, "id">): SpaceUrlCaptureProjection;
  updateUrlCapture(
    captureId: string,
    patch: Partial<
      Pick<
        SpaceUrlCaptureProjection,
        "status" | "downloadedBytes" | "totalBytes" | "processedMs" | "durationMs"
      >
    >,
  ): void;
  finishUrlCapture(captureId: string): void;
  addContent(data: SpaceNodePayload, position?: { x: number; y: number }): Promise<SpaceCanvasNode[]>;
  saveNodeContentForSpace(spaceId: string, change: NodePayloadChangeInput): Promise<void>;
}

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
  const projection = context.beginUrlCapture({
    spaceId,
    url: normalized,
    title: websiteDefaultTitle(normalized),
    position: target,
    status: "resolving",
    downloadedBytes: 0,
    totalBytes: null,
    processedMs: null,
    durationMs: null,
  });
  let retainFailure = false;
  try {
    const sourceKind = classifyUrlSource(new URL(normalized));
    void logInfo(`[spaces.url-capture] route=${sourceKind}`);
    if (context.getActiveSpaceId() !== spaceId) return undefined;
    if (sourceKind === "website") {
      return await addSavedWebsite({ ...context, activeSpaceId: spaceId }, normalized, target);
    }
    const outcome = await importMediaSource(context, {
      spaceId,
      url: normalized,
      target,
      captureId: projection.id,
    });
    retainFailure = outcome.failed;
    return outcome.node;
  } catch (error) {
    void logWarn(`URL capture failed for ${normalized}: ${formatErrorDiagnostic(error)}`);
    toast.error(i18n.global.t("spaces.importExport.website.captureFailed"));
    retainFailure = true;
    context.updateUrlCapture(projection.id, { status: "failed" });
    return undefined;
  } finally {
    if (!retainFailure) context.finishUrlCapture(projection.id);
  }
}

interface MediaSourceImportInput {
  spaceId: string;
  url: string;
  target: { x: number; y: number };
  captureId: string;
}

async function importMediaSource(context: UrlCaptureContext, input: MediaSourceImportInput) {
  try {
    const acquisition = await acquireMediaSource(input.url, (progress: MediaImportProgress) =>
      context.updateUrlCapture(input.captureId, {
        status: progress.stage,
        downloadedBytes: progress.downloadedBytes,
        totalBytes: progress.totalBytes,
        processedMs: progress.processedMs,
        durationMs: progress.durationMs,
      }),
    );
    if (context.getActiveSpaceId() !== input.spaceId) {
      if (acquisition.outcome === "imported") {
        try {
          await removeImportedMedia(acquisition.payload);
        } catch (error) {
          void logWarn(
            `Media import completed after navigation, but downloaded assets could not be removed: ${formatErrorDiagnostic(error)}`,
          );
        }
      }
      return { failed: false };
    }
    if (acquisition.outcome === "imported") {
      try {
        void logInfo(`[spaces.url-capture] media acquired; kind=${acquisition.payload.kind}; saving node`);
        const [node] = await context.addContent(acquisition.payload, input.target);
        void logInfo(`[spaces.url-capture] media node saved; created=${Boolean(node)}`);
        return { failed: false, node };
      } catch (error) {
        try {
          await removeImportedMedia(acquisition.payload);
        } catch (cleanupError) {
          throw new AggregateError(
            [error, cleanupError],
            "Media node could not be saved and downloaded assets could not be removed",
            { cause: error },
          );
        }
        throw error;
      }
    }
    throw new Error(`Media source is unsupported: ${input.url}`);
  } catch (error) {
    if (decodeAppError(error)?.code === "media_acquisition_cancelled") return { failed: false };
    void logWarn(`Media import failed for ${input.url}: ${formatErrorDiagnostic(error)}`);
    toast.error(i18n.global.t("spaces.media.import.failed"));
    context.updateUrlCapture(input.captureId, { status: "failed" });
    return { failed: true };
  }
}

async function removeImportedMedia(media: { assetPath: string; previewPath?: string | undefined }) {
  await removeSpaceAssets(
    [media.assetPath, media.previewPath].filter((path): path is string => path !== undefined),
  );
}
