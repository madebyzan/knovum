import { Channel } from "@tauri-apps/api/core";

import { invokeCommand } from "../../../../foundation/ipc/invoke-command";
import { importedMediaSchema, mediaImportProgressSchema, type MediaImportProgress } from "./schema/import";

interface MediaImportRequestInput {
  jobId: string;
  resourceTitle: string;
  url: string;
  onProgress: (progress: MediaImportProgress) => void;
}

export async function requestMediaImport({ jobId, resourceTitle, url, onProgress }: MediaImportRequestInput) {
  const channel = new Channel<unknown>();
  // Tauri Channel exposes its receiver as an onmessage property, not an EventTarget API.
  // oxlint-disable-next-line unicorn/prefer-add-event-listener
  channel.onmessage = (value) => onProgress(mediaImportProgressSchema.parse(value));
  const response = await invokeCommand<unknown>("plugin:spaces|import_media_from_url", {
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
  return invokeCommand("plugin:spaces|cancel_media_import", { jobId });
}
