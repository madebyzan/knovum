export type { SpacePackageProgress } from "../contracts/generated/SpacePackageProgress";
import { Channel } from "@tauri-apps/api/core";
import { open, save } from "@tauri-apps/plugin-dialog";
import { isString } from "es-toolkit";

import { invokeCommand } from "../../../foundation/ipc/invoke-command";
import type { SpacePackageProgress } from "../contracts/generated/SpacePackageProgress";
import type { Space } from "../model/space";
import { truncateUtf16 } from "../model/text";

interface SpacePackageImportContext {
  spaces: Space[];
  openSpace(id: string): Promise<void>;
}

interface SpacePackageExportContext {
  space: Pick<Space, "id" | "title">;
  commitActiveSession(): Promise<void>;
}

type ProgressListener = (progress: SpacePackageProgress) => void;

export async function exportSpacePackage(context: SpacePackageExportContext, onProgress?: ProgressListener) {
  const destination = await save({
    defaultPath: `${fileName(context.space.title)}.knovumspace`,
    filters: [{ name: "Knovum Space", extensions: ["knovumspace"] }],
  });
  if (!destination) return false;
  await context.commitActiveSession();
  await invokeCommand("plugin:spaces|export_space_package", {
    destinationPath: destination,
    spaceId: context.space.id,
    onProgress: progressChannel(onProgress),
  });
  return true;
}

export async function importSpacePackage(context: SpacePackageImportContext, onProgress?: ProgressListener) {
  const source = await open({
    multiple: false,
    directory: false,
    filters: [{ name: "Knovum Space", extensions: ["knovumspace"] }],
  });
  if (!isString(source)) return false;
  const space = await invokeCommand<Space>("plugin:spaces|import_space_package", {
    sourcePath: source,
    onProgress: progressChannel(onProgress),
  });
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
