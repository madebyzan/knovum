import { Channel, invoke } from "@tauri-apps/api/core";
import { open, save } from "@tauri-apps/plugin-dialog";
import { isString } from "es-toolkit";

import { removeSpaceAssets } from "../assets/asset-operations";
import type { Space } from "../model/space";
import { truncateUtf16 } from "../model/text";
import type { CanvasPackageCodec } from "./canvas-state-codec";
import type { ImportedSpacePackage } from "./contract";
import { parseSpacePackageManifest } from "./manifest-parser";
import { restoreSpacePackage } from "./restore";
import { collectSpacePackageSnapshot } from "./snapshot-transport";

interface SpacePackageImportContext {
  spaces: Space[];
  openSpace(id: string): Promise<void>;
}

interface SpacePackageExportContext {
  space: Pick<Space, "id" | "title">;
  commitActiveSession(): Promise<void>;
}

export interface SpacePackageProgress {
  stage: "writing" | "reading";
  processed: number;
  total: number;
  percent: number;
}

type ProgressListener = (progress: SpacePackageProgress) => void;

export async function exportSpacePackage(
  context: SpacePackageExportContext,
  canvasCodec: CanvasPackageCodec,
  onProgress?: ProgressListener,
) {
  const destination = await save({
    defaultPath: `${fileName(context.space.title)}.knovumspace`,
    filters: [{ name: "Knovum Space", extensions: ["knovumspace"] }],
  });
  if (!destination) return false;
  await context.commitActiveSession();
  const { manifest, assets } = await collectSpacePackageSnapshot(context.space.id);
  await invoke("plugin:spaces|export_space_package", {
    destinationPath: destination,
    manifest: currentManifestJson(manifest, canvasCodec),
    assets,
    onProgress: progressChannel(onProgress),
  });
  return true;
}

export async function importSpacePackage(
  context: SpacePackageImportContext,
  canvasCodec: CanvasPackageCodec,
  onProgress?: ProgressListener,
) {
  const source = await open({
    multiple: false,
    directory: false,
    filters: [{ name: "Knovum Space", extensions: ["knovumspace"] }],
  });
  if (!isString(source)) return false;
  const unpacked = await invoke<ImportedSpacePackage>("plugin:spaces|import_space_package", {
    sourcePath: source,
    onProgress: progressChannel(onProgress),
  });
  const space = await restoreSpacePackage(unpacked.manifest, unpacked.assets, canvasCodec).catch((error) =>
    cleanupImportedAssets(
      unpacked.assets.map((asset) => asset.path),
      error,
    ),
  );
  context.spaces.push(space);
  await context.openSpace(space.id);
  return true;
}

function progressChannel(listener?: ProgressListener) {
  const channel = new Channel<SpacePackageProgress>();
  // oxlint-disable-next-line unicorn/prefer-add-event-listener -- Tauri Channel exposes progress through onmessage, not EventTarget.
  if (listener) channel.onmessage = listener;
  return channel;
}

async function cleanupImportedAssets(paths: string[], cause: unknown): Promise<never> {
  try {
    await removeSpaceAssets(paths);
  } catch (cleanupError) {
    throw new AggregateError([cause, cleanupError], "Space package restore and asset cleanup both failed", {
      cause: cleanupError,
    });
  }
  throw cause;
}

function fileName(title: string) {
  const value = truncateUtf16(
    title
      .trim()
      .replace(/[\\/:*?"<>|]/gu, "-")
      .replace(/\s+/gu, " "),
    80,
  );
  if (!value) throw new Error("Space title cannot be used as a package file name");
  return value;
}

function currentManifestJson(manifest: unknown, canvasCodec: CanvasPackageCodec) {
  return JSON.stringify(parseSpacePackageManifest(JSON.stringify(manifest), canvasCodec));
}
