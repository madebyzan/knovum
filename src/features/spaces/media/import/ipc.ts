import { Channel } from "@tauri-apps/api/core";

import { invokeCommand } from "../../../../foundation/ipc/invoke-command";
import { importedMediaSchema, mediaImportProgressSchema, type MediaImportProgress } from "./schema/import";

export interface MediaImportRequestInput {
  jobId: string;
  resourceTitle: string;
  kind: "audio" | "video";
  url: string;
  onProgress: (progress: MediaImportProgress) => void;
}

/** Names the asset the import will publish into, so the canvas can reserve its node first. */
export function allocateMediaImportAsset(jobId: string, kind: "audio" | "video"): Promise<string> {
  return invokeCommand<string>("plugin:spaces|allocate_media_import_asset", { jobId, kind });
}

export async function requestMediaImport({
  jobId,
  resourceTitle,
  kind,
  url,
  onProgress,
}: MediaImportRequestInput) {
  const channel = new Channel<unknown>();
  // Tauri Channel exposes its receiver as an onmessage property, not an EventTarget API.
  // oxlint-disable-next-line unicorn/prefer-add-event-listener
  channel.onmessage = (value) => onProgress(mediaImportProgressSchema.parse(value));
  const response = await invokeCommand<unknown>("plugin:spaces|import_media_from_url", {
    jobId,
    resourceTitle,
    kind,
    url,
    onProgress: channel,
  });
  return response === null
    ? ({ outcome: "unsupported" } as const)
    : ({ outcome: "imported", media: importedMediaSchema.parse(response) } as const);
}

export function cancelMediaImport(jobId: string): Promise<void> {
  return invokeCommand("plugin:spaces|cancel_media_import", { jobId });
}
