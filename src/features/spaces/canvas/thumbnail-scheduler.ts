import { error as logError, warn as logWarning } from "@tauri-apps/plugin-log";

import { cleanupDeletedSpaceAssets, removeSpaceAsset } from "../assets/asset-operations";
import { saveCanvasThumbnailBlob } from "../assets/canvas-thumbnail-file";
import type { SpaceState } from "../session/session-state";
import { updateSpaceThumbnail } from "../workspace/space-operations";
import { useCanvasEditorSession } from "./editor-session";

const SETTLE_DELAY_MS = 800;
let pendingCapture: number | undefined;

export function scheduleCanvasThumbnail(state: SpaceState) {
  cancelCanvasThumbnail();
  const space = state.activeSpace.value;
  if (!space || (!state.thumbnailDirty.value && space.thumbnailPath)) return;
  pendingCapture = globalThis.setTimeout(() => {
    pendingCapture = undefined;
    void persistActiveCanvasThumbnail(state).catch((error) =>
      logError(`Failed to update Space thumbnail: ${String(error)}`),
    );
  }, SETTLE_DELAY_MS);
}

export function cancelCanvasThumbnail() {
  if (pendingCapture === undefined) return;
  globalThis.clearTimeout(pendingCapture);
  pendingCapture = undefined;
}

export async function commitActiveCanvasThumbnail(state: SpaceState) {
  cancelCanvasThumbnail();
  try {
    await persistActiveCanvasThumbnail(state);
  } catch (error) {
    await logWarning(`Space navigation continued without refreshing its derived thumbnail: ${String(error)}`);
  }
}

export async function persistActiveCanvasThumbnail(state: SpaceState) {
  const space = state.activeSpace.value;
  if (!space || (!state.thumbnailDirty.value && space.thumbnailPath)) return;
  if (!state.spaces.value.some((candidate) => candidate.id === space.id)) return;
  const editor = useCanvasEditorSession().activeEditor.value;
  if (!editor) throw new Error("Space canvas is not mounted");
  state.thumbnailDirty.value = false;
  let stagedPath: string | undefined;
  try {
    stagedPath = await saveCanvasThumbnailBlob(await editor.captureCanvasOverview());
    if (state.activeSpace.value?.id !== space.id) {
      await removeSpaceAsset(stagedPath);
      return;
    }
    const obsoletePaths = await updateSpaceThumbnail(space.id, stagedPath);
    space.thumbnailPath = stagedPath;
    stagedPath = undefined;
    await cleanupDeletedSpaceAssets(obsoletePaths);
  } catch (error) {
    if (stagedPath) await removeSpaceAsset(stagedPath);
    state.thumbnailDirty.value = true;
    throw error;
  }
}
