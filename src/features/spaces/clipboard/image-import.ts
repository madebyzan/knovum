import { invoke } from "@tauri-apps/api/core";

import type { ImportedSpaceAsset } from "../assets/imported-asset";

const MAX_PNG_BYTES = 16 * 1024 * 1024;

interface PersistedClipboardImage {
  path: string;
  sourceRevision: string;
}

// An empty or file-only clipboard is a normal paste outcome, so "nothing to import" is reported as
// `undefined` rather than thrown; only a real import failure raises.
export async function importSpaceClipboardImage(name: string): Promise<ImportedSpaceAsset | undefined> {
  if (!(await invoke<boolean>("plugin:clipboard|has_image"))) return undefined;
  const bytes = await invoke<number[]>("plugin:clipboard|read_image_binary");
  if (!bytes.length) return undefined;
  if (bytes.length > MAX_PNG_BYTES) throw new Error("Clipboard image exceeds the 16 MiB asset limit");
  const persisted = await invoke<PersistedClipboardImage>("plugin:spaces|persist_clipboard_image", {
    bytes,
  });
  return { kind: "image", name, path: persisted.path, sourceRevision: persisted.sourceRevision };
}

export function clipboardContainsImage(event: ClipboardEvent) {
  return [...(event.clipboardData?.items ?? [])].some((item) => item.type.startsWith("image/"));
}
