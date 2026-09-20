import { invoke } from "@tauri-apps/api/core";

export async function saveCanvasThumbnailBlob(blob: Blob): Promise<string> {
  if (blob.type !== "image/jpeg") throw new Error("Canvas thumbnail must be a JPEG image");
  const bytes = Array.from(new Uint8Array(await blob.arrayBuffer()));
  return invoke<string>("plugin:spaces|persist_canvas_thumbnail", { bytes });
}
