import { Channel } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { error as logError } from "@tauri-apps/plugin-log";
import { isString, uniq } from "es-toolkit";

import { invokeCommand } from "../../../foundation/ipc/invoke-command";
import type { KnowledgeLocator } from "../knowledge/model/schema";
import type { SpaceNodePayload } from "../model/nodes/node-payload";
import type { ImportedSpaceAsset } from "./imported-asset";

const resourceRefreshes = new Map<string, Promise<SpaceNodePayload | undefined>>();

export async function chooseSpaceSourcePath(): Promise<string | undefined> {
  const source = await open({ multiple: false, directory: false });
  return isString(source) ? source : undefined;
}

export interface PreparedSpaceImport {
  jobId: string | null;
  asset: ImportedSpaceAsset;
}

export interface SpaceAssetImportProgress {
  stage: "copying";
  bytesCopied: number;
  totalBytes?: number;
}

export type SpaceImportSourceAccess = "scoped" | "clipboard";

export function prepareSpaceAssetImport(
  sourcePath: string,
  sourceAccess: SpaceImportSourceAccess,
): Promise<PreparedSpaceImport> {
  return invokeCommand("plugin:spaces|stage_asset_import", { sourcePath, sourceAccess });
}

export function prepareAgentArtifactImport(
  spaceId: string,
  sessionId: string,
  relativePath: string,
): Promise<PreparedSpaceImport> {
  return invokeCommand("plugin:spaces|stage_generated_artifact_import", { spaceId, sessionId, relativePath });
}

export function copyStagedSpaceAsset(
  jobId: string,
  onProgress: (progress: SpaceAssetImportProgress) => void,
): Promise<ImportedSpaceAsset> {
  const channel = new Channel<SpaceAssetImportProgress>();
  // oxlint-disable-next-line unicorn/prefer-add-event-listener -- Tauri Channel exposes progress through onmessage, not EventTarget.
  channel.onmessage = onProgress;
  return invokeCommand("plugin:spaces|commit_asset_import", { jobId, onProgress: channel });
}

export function cancelSpaceAssetImport(jobId: string): Promise<void> {
  return invokeCommand("plugin:spaces|cancel_asset_import", { jobId });
}

export function deriveSpaceAssetContent(
  spaceId: string,
  nodeId: string,
  jobId: string,
  onUpdate: (data: SpaceNodePayload) => void,
): Promise<SpaceNodePayload> {
  const channel = new Channel<SpaceNodePayload>();
  // oxlint-disable-next-line unicorn/prefer-add-event-listener -- Tauri Channel exposes progress through onmessage, not EventTarget.
  channel.onmessage = onUpdate;
  return invokeCommand<SpaceNodePayload>("plugin:spaces|enrich_asset", {
    spaceId,
    nodeId,
    jobId,
    onUpdate: channel,
  });
}

export function cancelSpaceAssetProcessing(jobId: string): Promise<void> {
  return invokeCommand("plugin:spaces|cancel_asset_enrichment", { jobId });
}

export function refreshSpaceAsset(spaceId: string, nodeId: string): Promise<SpaceNodePayload | undefined> {
  const key = `${spaceId}:${nodeId}`;
  const pending = resourceRefreshes.get(key);
  if (pending) return pending;
  const request = invokeCommand<SpaceNodePayload | null>("plugin:spaces|refresh_asset", { spaceId, nodeId })
    .then((data) => data ?? undefined)
    .finally(() => resourceRefreshes.delete(key));
  resourceRefreshes.set(key, request);
  return request;
}

export function openSpaceAsset(sourcePath: string): Promise<void> {
  return invokeCommand("plugin:spaces|open_asset", { sourcePath });
}

export function openSpaceAssetAtLocation(sourcePath: string, locator: KnowledgeLocator): Promise<void> {
  const location = externalLocation(locator);
  if (!location) return Promise.reject(new Error("This source location has no document previewer"));
  return invokeCommand("plugin:spaces|open_asset_at_location", { sourcePath, location });
}

function externalLocation(locator: KnowledgeLocator) {
  const location = locator.location;
  if (location.kind === "word") return { kind: "word", quote: locator.quote?.exact };
  if (location.kind === "slides") return { kind: "slides", slide: location.slide };
  if (location.kind === "sheet") return { kind: "sheet", sheet: location.sheet, range: location.range };
  return undefined;
}

export function duplicateSpaceAsset(sourcePath: string): Promise<string> {
  return invokeCommand("plugin:spaces|duplicate_asset", { sourcePath });
}

export function removeSpaceAsset(sourcePath: string): Promise<void> {
  return removeSpaceAssets([sourcePath]);
}

export function removeSpaceAssets(paths: readonly string[]): Promise<void> {
  const sourcePaths = uniq(paths);
  if (!sourcePaths.length) return Promise.resolve();
  return invokeCommand("plugin:spaces|remove_assets", { sourcePaths });
}

export async function cleanupDeletedSpaceAssets(paths: readonly string[]): Promise<void> {
  if (!paths.length) return;
  try {
    await removeSpaceAssets(paths);
  } catch (error) {
    void logError(`Space database deletion committed, but local asset cleanup failed: ${String(error)}`);
    throw new Error("Space data was deleted, but local asset cleanup failed", { cause: error });
  }
}
