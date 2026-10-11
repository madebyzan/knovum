import { error as logError } from "@tauri-apps/plugin-log";
import pLimit from "p-limit";

import { formatErrorDiagnostic } from "../../../foundation/errors/error-format";
import { cleanupDeletedSpaceAssets, removeSpaceAsset } from "../assets/asset-operations";
import { saveCanvasThumbnailBlob } from "../assets/canvas-thumbnail-file";
import type { Space } from "../model/space";
import type { SpaceState } from "../session/session-state";
import { updateSpaceThumbnail } from "../workspace/space-operations";
import { currentCanvasEditor } from "./editor-session";

const publications = pLimit(1);
const captureBySpaceId = new Map<string, symbol>();
function needsCapture(state: SpaceState, space: Space) {
  return state.thumbnailDirty.value || (!space.thumbnailPath && !captureBySpaceId.has(space.id));
}

export function captureActiveCanvasThumbnail(state: SpaceState) {
  void persistActiveCanvasThumbnail(state).catch((error) =>
    logError(`Failed to update Space thumbnail: ${formatErrorDiagnostic(error)}`),
  );
}

async function persistActiveCanvasThumbnail(state: SpaceState) {
  const space = state.activeSpace.value;
  if (!space || !needsCapture(state, space)) return;
  if (!state.spaces.value.some((candidate) => candidate.id === space.id)) return;
  const editor = currentCanvasEditor();
  if (!editor) throw new Error("Space canvas is not mounted");
  const capture = Symbol(space.id);
  captureBySpaceId.set(space.id, capture);
  const snapshot = state.editSnapshot.value;
  const nodes = state.nodes.value;
  const updatedAt = space.updatedAt;
  state.thumbnailDirty.value = false;
  let stagedPath: string | undefined;
  try {
    stagedPath = await saveCanvasThumbnailBlob(await editor.captureCanvasOverview());
    await publications(async () => {
      const current = state.spaces.value.find((candidate) => candidate.id === space.id);
      const changed =
        state.activeSpace.value?.id === space.id &&
        (state.editSnapshot.value !== snapshot || state.nodes.value !== nodes);
      if (changed) state.thumbnailDirty.value = true;
      if (
        !current ||
        changed ||
        current.updatedAt !== updatedAt ||
        captureBySpaceId.get(space.id) !== capture
      ) {
        await removeSpaceAsset(stagedPath!);
        stagedPath = undefined;
        return;
      }
      const obsoletePaths = await updateSpaceThumbnail(space.id, stagedPath!);
      current.thumbnailPath = stagedPath!;
      stagedPath = undefined;
      await cleanupDeletedSpaceAssets(obsoletePaths);
    });
  } catch (error) {
    if (state.activeSpace.value?.id === space.id) state.thumbnailDirty.value = true;
    await removeFailedThumbnail(stagedPath, error);
    throw error;
  } finally {
    if (captureBySpaceId.get(space.id) === capture) captureBySpaceId.delete(space.id);
  }
}

async function removeFailedThumbnail(stagedPath: string | undefined, error: unknown) {
  if (!stagedPath) return;
  try {
    await removeSpaceAsset(stagedPath);
  } catch (cleanupError) {
    throw new AggregateError(
      [error, cleanupError],
      "Space thumbnail update failed and its staged asset could not be removed",
      { cause: error },
    );
  }
}
