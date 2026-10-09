import { warn as logWarn } from "@tauri-apps/plugin-log";
import { compact } from "es-toolkit";

import { formatErrorDiagnostic } from "../../../foundation/errors/error-format";
import { removeSpaceAssets } from "../assets/asset-operations";
import type { SpaceWebsitePayload } from "../model/nodes/node-payload";
import type { NodePayloadChangeInput } from "../nodes/node-operations";

export interface SavedWebsiteBinding {
  context: {
    saveNodeContentForSpace(spaceId: string, change: NodePayloadChangeInput): Promise<void>;
  };
  spaceId: string;
  id: string;
  data: SpaceWebsitePayload;
}

export async function persistSavedWebsite(
  target: SavedWebsiteBinding,
  paths: readonly (string | undefined)[],
  metadata: Partial<Omit<SpaceWebsitePayload, "kind">>,
  explicitPreviewPath?: string,
) {
  const cachedPaths = compact(paths);
  const [faviconPath, previewImagePath] = paths;
  const nextPreviewPath = explicitPreviewPath ?? previewImagePath;
  const previous = clearedWebsiteMetadata(target.data);
  const next: SpaceWebsitePayload = { ...previous, ...metadata };
  if (faviconPath && !explicitPreviewPath) next.faviconPath = faviconPath;
  if (nextPreviewPath) next.previewImagePath = nextPreviewPath;
  const stalePaths = compact([target.data.faviconPath, target.data.previewImagePath]).filter(
    (path) => path !== next.faviconPath && path !== next.previewImagePath,
  );
  try {
    await saveWebsiteBinding(target, next);
  } catch (error) {
    try {
      await removeSpaceAssets(cachedPaths);
    } catch (cleanupError) {
      throw new AggregateError(
        [error, cleanupError],
        "Website metadata could not be saved and cached assets could not be removed",
        { cause: error },
      );
    }
    throw error;
  }
  try {
    await removeSpaceAssets(stalePaths);
  } catch (error) {
    void logWarn(`Website asset reclamation failed for ${target.id}: ${formatErrorDiagnostic(error)}`);
  }
}

export async function markSavedWebsiteFailedAfter(target: SavedWebsiteBinding, error: unknown) {
  try {
    await saveWebsiteBinding(target, { ...target.data, metadataState: "failed" });
  } catch (projectionError) {
    throw new AggregateError(
      [error, projectionError],
      "Website hydration failed and its failure state could not be saved",
      { cause: error },
    );
  }
}

async function saveWebsiteBinding(target: SavedWebsiteBinding, payload: SpaceWebsitePayload) {
  await target.context.saveNodeContentForSpace(target.spaceId, {
    id: target.id,
    expectedData: target.data,
    data: payload,
  });
  target.data = payload;
}

function clearedWebsiteMetadata(data: SpaceWebsitePayload): SpaceWebsitePayload {
  const previous = { ...data };
  delete previous.siteName;
  delete previous.description;
  delete previous.contentPreview;
  delete previous.websiteSections;
  delete previous.canonicalUrl;
  delete previous.faviconPath;
  delete previous.previewImagePath;
  return previous;
}
