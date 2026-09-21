import { isAbsolute, resolve, sep } from "@tauri-apps/api/path";

import { formatOptionalBytes } from "../../../foundation/format/bytes";
import { i18n } from "../../../foundation/i18n/i18n";
import { useTaskStore } from "../../../foundation/tasks/task-store";
import {
  cancelSpaceAssetImport,
  copyStagedSpaceAsset,
  prepareSpaceAssetImport,
  removeSpaceAssets,
} from "../assets/asset-operations";
import type { ImportedSpaceAsset } from "../assets/imported-asset";
import { importedSpaceNodePayload } from "../assets/imported-asset-payload";
import { createSpaceNode } from "../canvas/node";
import { noteFromPlainText } from "../canvas/text-import";
import type { SpaceNodePayload } from "../model/nodes/node-payload";
import { normalizeSpaceUrl, websiteDefaultTitle } from "../website/url";
import type { CanvasJsonImportRequest, JsonCanvasImportNodeV1 } from "./json-canvas-import-contract";
import type { PreparedJsonCanvasNode } from "./json-canvas-import-repository";

export interface JsonCanvasNodePreparation {
  nodes: PreparedJsonCanvasNode[];
  assetPaths: string[];
  backgroundPaths: Map<string, string>;
  taskIds: string[];
}

export async function prepareJsonCanvasNodes(
  request: CanvasJsonImportRequest,
): Promise<JsonCanvasNodePreparation> {
  const preparation: JsonCanvasNodePreparation = {
    nodes: [],
    assetPaths: [],
    backgroundPaths: new Map(),
    taskIds: [],
  };
  const nodeIds = new Map(request.plan.nodes.map(({ sourceId }) => [sourceId, crypto.randomUUID()]));
  try {
    for (const source of request.plan.nodes) {
      // oxlint-disable-next-line no-await-in-loop -- Serialization keeps every committed asset cleanup-addressable.
      const data = await nodePayload(source, request.sourceDirectory, preparation);
      const sectionId = source.parentSourceId ? requiredNodeId(nodeIds, source.parentSourceId) : undefined;
      preparation.nodes.push({
        sourceId: source.sourceId,
        node: createSpaceNode(
          { ...data, ...(sectionId ? { sectionId } : {}) } as SpaceNodePayload,
          { x: source.frame.x, y: source.frame.y },
          requiredNodeId(nodeIds, source.sourceId),
        ),
      });
    }
    return preparation;
  } catch (error) {
    return discardJsonCanvasPreparation(preparation, error);
  }
}

export function completeJsonCanvasPreparation(preparation: JsonCanvasNodePreparation) {
  const tasks = useTaskStore();
  for (const taskId of preparation.taskIds) tasks.succeed(taskId);
}

export async function discardJsonCanvasPreparation(
  preparation: JsonCanvasNodePreparation,
  error: unknown,
): Promise<never> {
  const tasks = useTaskStore();
  for (const taskId of preparation.taskIds) tasks.fail(taskId, String(error));
  try {
    await removeSpaceAssets(preparation.assetPaths);
  } catch (cleanupError) {
    throw new AggregateError([error, cleanupError], "JSON Canvas import and asset cleanup failed", {
      cause: cleanupError,
    });
  }
  throw error;
}

async function nodePayload(
  source: JsonCanvasImportNodeV1,
  sourceDirectory: string,
  preparation: JsonCanvasNodePreparation,
): Promise<SpaceNodePayload> {
  const content = source.content;
  if (content.type === "text") return noteFromPlainText(content.text, i18n.global.t("spaces.editor.newNote"));
  if (content.type === "link") return websitePayload(content.url);
  if (content.type === "group") {
    const background = source.appearance?.background;
    if (background) {
      const asset = await importCanvasResource(source.sourceId, sourceDirectory, background, preparation);
      if (!asset.path) throw new Error(`JSON Canvas group ${source.sourceId} background has no managed path`);
      preparation.backgroundPaths.set(source.sourceId, asset.path);
    }
    return {
      kind: "section",
      title: content.label?.trim() || i18n.global.t("spaces.canvas.untitledSection"),
    };
  }
  return importFilePayload(source.sourceId, sourceDirectory, content.file, preparation);
}

function websitePayload(value: string): SpaceNodePayload {
  const url = normalizeSpaceUrl(value);
  if (!url) throw new Error(`JSON Canvas link is not an HTTP URL: ${value}`);
  const title = websiteDefaultTitle(url);
  return { kind: "website", title, url, siteName: title, metadataState: "idle" };
}

async function importFilePayload(
  sourceId: string,
  sourceDirectory: string,
  relativePath: string,
  preparation: JsonCanvasNodePreparation,
) {
  return importedSpaceNodePayload(
    await importCanvasResource(sourceId, sourceDirectory, relativePath, preparation),
  );
}

async function importCanvasResource(
  sourceId: string,
  sourceDirectory: string,
  relativePath: string,
  preparation: JsonCanvasNodePreparation,
): Promise<ImportedSpaceAsset> {
  const sourcePath = await resolveCanvasFile(sourceId, sourceDirectory, relativePath);
  const tasks = useTaskStore();
  const taskId = tasks.start(i18n.global.t("spaces.transfer.importingItem"));
  preparation.taskIds.push(taskId);
  const prepared = await prepareSpaceAssetImport(sourcePath, "scoped");
  tasks.title(taskId, prepared.asset.name);
  const jobId = prepared.jobId;
  if (!jobId) return prepared.asset;
  tasks.setCancellation(taskId, () => cancelSpaceAssetImport(jobId));
  try {
    const asset = await copyStagedSpaceAsset(jobId, (progress) => updateProgress(taskId, progress));
    tasks.setCancellation(taskId);
    if (asset.path) preparation.assetPaths.push(asset.path);
    if (asset.previewPath) preparation.assetPaths.push(asset.previewPath);
    return asset;
  } catch (error) {
    await cancelSpaceAssetImport(jobId);
    throw error;
  }
}

function updateProgress(taskId: string, progress: { bytesCopied: number; totalBytes?: number }) {
  const tasks = useTaskStore();
  const current = formatOptionalBytes(progress.bytesCopied) ?? "";
  if (!progress.totalBytes) tasks.detail(taskId, current);
  else {
    tasks.progress(
      taskId,
      (progress.bytesCopied / progress.totalBytes) * 100,
      `${current} / ${formatOptionalBytes(progress.totalBytes)}`,
    );
  }
}

async function resolveCanvasFile(sourceId: string, sourceDirectory: string, relativePath: string) {
  if (await isAbsolute(relativePath))
    throw new Error(`JSON Canvas resource ${sourceId} must use a relative path`);
  const directory = await resolve(sourceDirectory);
  const path = await resolve(directory, relativePath);
  if (path !== directory && !path.startsWith(`${directory}${sep()}`)) {
    throw new Error(`JSON Canvas resource ${sourceId} escapes the canvas directory`);
  }
  return path;
}

function requiredNodeId(ids: ReadonlyMap<string, string>, sourceId: string) {
  const nodeId = ids.get(sourceId);
  if (!nodeId) throw new Error(`JSON Canvas node ${sourceId} is unavailable`);
  return nodeId;
}
