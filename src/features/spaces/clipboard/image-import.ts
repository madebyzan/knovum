import { hasImage, readImageBinary } from "tauri-plugin-clipboard-api";

import { invokeCommand } from "../../../foundation/ipc/invoke-command";
import type { PreparedAsset as ImportedSpaceAsset } from "../contracts/generated/PreparedAsset";

const MAX_PNG_BYTES = 16 * 1024 * 1024;

interface PersistedClipboardImage {
  path: string;
  sourceRevision: string;
}

// An empty or file-only clipboard is a normal paste outcome, so "nothing to import" is reported as
// `undefined` rather than thrown; only a real import failure raises.
export async function importSpaceClipboardImage(name: string): Promise<ImportedSpaceAsset | undefined> {
  if (!(await hasImage())) return undefined;
  const bytes = (await readImageBinary("Uint8Array")) as Uint8Array;
  if (!bytes.byteLength) return undefined;
  if (bytes.byteLength > MAX_PNG_BYTES) throw new Error("Clipboard image exceeds the 16 MiB asset limit");
  const persisted = await invokeCommand<PersistedClipboardImage>("plugin:spaces|persist_clipboard_image", {
    bytes: Array.from(bytes),
  });
  return { kind: "image", name, path: persisted.path, sourceRevision: persisted.sourceRevision };
}

export function clipboardContainsImage(event: ClipboardEvent) {
  return [...(event.clipboardData?.items ?? [])].some((item) => item.type.startsWith("image/"));
}
