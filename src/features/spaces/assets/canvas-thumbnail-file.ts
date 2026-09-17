import { appDataDir, join } from "@tauri-apps/api/path";
import { BaseDirectory, mkdir, writeFile } from "@tauri-apps/plugin-fs";

const DIRECTORY = "spaces/assets";

export async function saveCanvasThumbnailBlob(blob: Blob): Promise<string> {
  if (blob.type !== "image/jpeg") throw new Error("Canvas thumbnail must be a JPEG image");
  const name = `canvas-thumbnail-${crypto.randomUUID()}.jpg`;
  await mkdir(DIRECTORY, { baseDir: BaseDirectory.AppData, recursive: true });
  await writeFile(`${DIRECTORY}/${name}`, new Uint8Array(await blob.arrayBuffer()), {
    baseDir: BaseDirectory.AppData,
  });
  return join(await appDataDir(), DIRECTORY, name);
}
