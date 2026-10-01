import type { SpaceCanvasNode } from "../model/nodes/canvas-node";
import type { SpaceNodePayload } from "../model/nodes/node-payload";
import { removeSpaceAsset } from "./asset-operations";
import type { ImportedSpaceAsset } from "./imported-asset";
import { importedSpaceNodePayload } from "./imported-asset-payload";

type AddImportedContent = (
  data: SpaceNodePayload,
  position: { x: number; y: number },
) => Promise<SpaceCanvasNode[]>;

export async function addImportedSpaceAsset(
  asset: ImportedSpaceAsset,
  position: { x: number; y: number },
  addContent: AddImportedContent,
) {
  const data = importedSpaceNodePayload(asset);
  try {
    return await addContent(data, position);
  } catch (error) {
    if (!asset.path) throw error;
    try {
      await removeSpaceAsset(asset.path);
    } catch (cleanupError) {
      throw new AggregateError([error, cleanupError], "Space import failed and staged asset cleanup failed", {
        cause: error,
      });
    }
    throw error;
  }
}
