import { Channel } from "@tauri-apps/api/core";

import { invokeCommand } from "../../../../foundation/ipc/invoke-command";
import {
  importedMediaSchema,
  mediaAcquisitionEventSchema,
  type MediaAcquisitionEvent,
} from "./schema/import";

interface MediaImportRequestInput {
  jobId: string;
  resourceTitle: string;
  url: string;
  onEvent: (event: MediaAcquisitionEvent) => void;
}

export async function requestMediaImport({ jobId, resourceTitle, url, onEvent }: MediaImportRequestInput) {
  const channel = new Channel<unknown>();
  // Tauri Channel exposes its receiver as an onmessage property, not an EventTarget API.
  // oxlint-disable-next-line unicorn/prefer-add-event-listener
  channel.onmessage = (value) => onEvent(mediaAcquisitionEventSchema.parse(value));
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
