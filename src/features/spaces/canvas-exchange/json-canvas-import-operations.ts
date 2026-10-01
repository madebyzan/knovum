import { Channel } from "@tauri-apps/api/core";

import { invokeCommand } from "../../../foundation/ipc/invoke-command";
import type { JsonCanvasImportReceipt } from "../contracts/generated/JsonCanvasImportReceipt";
import { invokeConfirmedMutation } from "../structure/confirmed-mutation";

interface CanvasJsonImportInput {
  spaceId: string;
  sourcePath: string;
  mutationId: string;
  baseRevision: number;
  noteTitle: string;
  sectionTitle: string;
}

interface JsonCanvasImportProgress {
  stage: string;
  bytesCopied: number;
  totalBytes?: number;
}

export function importJsonCanvas(
  input: CanvasJsonImportInput,
  onProgress: (progress: JsonCanvasImportProgress) => void,
) {
  const channel = new Channel<JsonCanvasImportProgress>();
  // oxlint-disable-next-line unicorn/prefer-add-event-listener -- Tauri Channel exposes onmessage rather than EventTarget.
  channel.onmessage = onProgress;
  return invokeConfirmedMutation<JsonCanvasImportReceipt>(
    "plugin:spaces|import_json_canvas_file",
    { request: input, onProgress: channel },
    input.mutationId,
  );
}

export function cancelJsonCanvasImport(mutationId: string) {
  return invokeCommand<void>("plugin:spaces|cancel_json_canvas_import", { mutationId });
}
