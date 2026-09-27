import { Channel } from "@tauri-apps/api/core";

import { invokeCommand as invoke } from "../../../../foundation/ipc/invoke-command";
import { importedMediaSchema, mediaImportProgressSchema, type MediaImportProgress } from "./contract";

export async function requestMediaImport(
  jobId: string,
  resourceTitle: string,
  url: string,
  onProgress: (progress: MediaImportProgress) => void,
) {
  const channel = new Channel<unknown>();
  // Tauri Channel exposes its receiver as an onmessage property, not an EventTarget API.
  // oxlint-disable-next-line unicorn/prefer-add-event-listener
  channel.onmessage = (value) => onProgress(mediaImportProgressSchema.parse(value));
  const response = await invoke<unknown>("plugin:spaces|import_media_from_url", {
    jobId,
    resourceTitle,
    url,
    onProgress: channel,
  });
  return response === null
    ? ({ outcome: "unsupported" } as const)
    : ({ outcome: "imported", media: importedMediaSchema.parse(response) } as const);
}

export function cancelMediaImport(jobId: string): Promise<void> {
  return invoke("plugin:spaces|cancel_media_import", { jobId });
}
