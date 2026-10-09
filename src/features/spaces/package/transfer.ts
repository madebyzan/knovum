import { Channel } from "@tauri-apps/api/core";
import { open, save } from "@tauri-apps/plugin-dialog";
import { isString } from "es-toolkit";

import { invokeCommand } from "../../../foundation/ipc/invoke-command";
import type { SpacePackageProgress } from "../contracts/generated/SpacePackageProgress";
import type { Space } from "../model/space";
import { truncateUtf16 } from "../model/text";

interface SpacePackageImportContext {
  spaces: Space[];
  openSpace(spaceId: string): Promise<void>;
}

interface SpacePackageExportContext {
  space: Pick<Space, "id" | "title">;
}

type ProgressListener = (progress: SpacePackageProgress) => void;

export interface SpacePackageTransfer {
  admitted: Promise<void>;
  completion: Promise<void>;
}

export async function chooseSpacePackageExportDestination(title: string) {
  const destination = await save({
    defaultPath: `${fileName(title)}.knovumspace`,
    filters: [{ name: "Knovum Space", extensions: ["knovumspace"] }],
  });
  return destination || undefined;
}

export function exportSpacePackage(
  context: SpacePackageExportContext,
  destination: string,
  operationId: string,
  onProgress?: ProgressListener,
): SpacePackageTransfer {
  const admission = admissionChannel(onProgress);
  const completion = invokeCommand<void>("plugin:spaces|export_space_package", {
    destinationPath: destination,
    operationId,
    spaceId: context.space.id,
    onProgress: admission.channel,
  });
  completion.then(
    () => {
      admission.rejectBeforeAdmission();
      return undefined;
    },
    (error) => {
      admission.reject(error);
      return undefined;
    },
  );
  return { admitted: admission.promise, completion };
}

export async function chooseSpacePackageImportSource() {
  const source = await open({
    multiple: false,
    directory: false,
    filters: [{ name: "Knovum Space", extensions: ["knovumspace"] }],
  });
  return isString(source) ? source : undefined;
}

export function importSpacePackage(
  context: SpacePackageImportContext,
  source: string,
  operationId: string,
  onProgress?: ProgressListener,
): SpacePackageTransfer {
  const admission = admissionChannel(onProgress);
  const completion = invokeCommand<Space>("plugin:spaces|import_space_package", {
    operationId,
    sourcePath: source,
    onProgress: admission.channel,
  }).then(async (space) => {
    context.spaces.push(space);
    await context.openSpace(space.id);
    return undefined;
  });
  completion.then(
    () => {
      admission.rejectBeforeAdmission();
      return undefined;
    },
    (error) => {
      admission.reject(error);
      return undefined;
    },
  );
  return { admitted: admission.promise, completion };
}

export function cancelSpacePackageTransfer(operationId: string) {
  return invokeCommand("plugin:spaces|cancel_space_package_transfer", { operationId });
}

function admissionChannel(listener?: ProgressListener) {
  const channel = new Channel<SpacePackageProgress>();
  let admitted = false;
  let resolveAdmission!: () => void;
  let rejectAdmission!: (error: unknown) => void;
  const promise = new Promise<void>((resolve, reject) => {
    resolveAdmission = resolve;
    rejectAdmission = reject;
  });
  void promise.catch(() => undefined);
  // oxlint-disable-next-line unicorn/prefer-add-event-listener -- Tauri Channel exposes progress through onmessage, not EventTarget.
  channel.onmessage = (progress) => {
    if (progress.stage === "admitted") {
      admitted = true;
      resolveAdmission();
      return;
    }
    listener?.(progress);
  };
  return {
    channel,
    promise,
    reject: (error: unknown) => {
      if (!admitted) rejectAdmission(error);
    },
    rejectBeforeAdmission: () => {
      if (!admitted) {
        rejectAdmission(new Error("Space package transfer completed before admission"));
      }
    },
  };
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
