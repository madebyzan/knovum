import { platform } from "@tauri-apps/plugin-os";

import { invokeCommand } from "../../../foundation/ipc/invoke-command";

/** System preview is backed by Quick Look on macOS and the Shell preview host on Windows. */
export function isAssetPreviewAvailable(): boolean {
  return platform() === "macos" || platform() === "windows";
}

/**
 * Previews a file in the system panel. The path must be a Space-owned asset: Rust re-authorizes it
 * against the private asset directory, so the WebView cannot turn this into a general file reader.
 */
export function previewSpaceAsset(path: string): Promise<void> {
  if (!isAssetPreviewAvailable()) {
    return Promise.reject(new Error("System preview is unavailable on this platform"));
  }
  return invokeCommand("plugin:spaces|preview_asset", { sourcePath: path });
}
