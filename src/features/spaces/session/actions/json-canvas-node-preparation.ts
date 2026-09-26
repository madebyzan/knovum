import { isAbsolute, resolve, sep } from "@tauri-apps/api/path";

import { i18n } from "../../../../foundation/i18n/i18n";
import { useTaskStore } from "../../../../foundation/tasks/task-store";
import {
  cancelSpaceAssetImport,
  copyStagedSpaceAsset,
  prepareSpaceAssetImport,
  removeSpaceAssets,
} from "../../assets/asset-operations";
import { formatByteSize } from "../../assets/format-byte-size";
import type { ImportedSpaceAsset } from "../../assets/imported-asset";
import { importedSpaceNodePayload } from "../../assets/imported-asset-payload";
import type {
  CanvasJsonImportRequest,
  JsonCanvasImportContentV1,
} from "../../canvas-exchange/json-canvas-import-contract";
import type { PreparedJsonCanvasNode } from "../../canvas-exchange/json-canvas-import-operations";
import { createSpaceNode } from "../../canvas/node";
import { noteFromPlainText } from "../../canvas/text-import";
import type { SpaceNodePayload } from "../../model/nodes/node-payload";
import { normalizeSpaceUrl, websiteDefaultTitle } from "../../website/url";

export interface JsonCanvasNodePreparation {
  nodes: PreparedJsonCanvasNode[];
  contentIds: Map<string, string>;
  sectionIds: Map<string, string>;
  assetPaths: string[];
  backgroundPaths: Map<string, string>;
  taskIds: string[];
}

export async function prepareJsonCanvasNodes(
  request: CanvasJsonImportRequest,
): Promise<JsonCanvasNodePreparation> {
  const preparation: JsonCanvasNodePreparation = {
    nodes: [],
    contentIds: new Map(request.plan.contents.map(({ sourceId }) => [sourceId, crypto.randomUUID()])),
    sectionIds: new Map(request.plan.sections.map(({ sourceId }) => [sourceId, crypto.randomUUID()])),
    assetPaths: [],
    backgroundPaths: new Map(),
    taskIds: [],
  };
  try {
    await runInOrder(request.plan.sections, async (source) => {
      const background = source.appearance?.background;
      if (!background) return;
      const asset = await importCanvasResource(
        source.sourceId,
        request.sourceDirectory,
        background,
        preparation,
      );
      if (!asset.path) throw new Error(`JSON Canvas group ${source.sourceId} background has no managed path`);
      preparation.backgroundPaths.set(source.sourceId, asset.path);
    });
    await runInOrder(request.plan.contents, async (source) => {
      const data = await nodePayload(source, request.sourceDirectory, preparation);
      preparation.nodes.push({
        sourceId: source.sourceId,
        node: createSpaceNode(
          data,
          { x: source.frame.x, y: source.frame.y },
          requiredNodeId(preparation.contentIds, source.sourceId),
        ),
      });
    });
    return preparation;
  } catch (error) {
    return discardJsonCanvasPreparation(preparation, error);
  }
}

function runInOrder<T>(values: readonly T[], operation: (value: T) => Promise<void>) {
  return values.reduce((previous, value) => previous.then(() => operation(value)), Promise.resolve());
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
  source: JsonCanvasImportContentV1,
  sourceDirectory: string,
  preparation: JsonCanvasNodePreparation,
): Promise<SpaceNodePayload> {
  const content = source.content;
  if (content.type === "text")
    return noteFromPlainText(content.text, i18n.global.t("spaces.editor.note.new"));
  if (content.type === "link") return websitePayload(content.url);
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
  const taskId = tasks.start(i18n.global.t("spaces.importExport.progress.resource"));
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
  const current = formatByteSize(progress.bytesCopied) ?? "";
  if (!progress.totalBytes) tasks.detail(taskId, current);
  else {
    tasks.progress(
      taskId,
      (progress.bytesCopied / progress.totalBytes) * 100,
      `${current} / ${formatByteSize(progress.totalBytes)}`,
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
