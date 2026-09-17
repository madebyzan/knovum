import { invoke } from "@tauri-apps/api/core";
import { platform } from "@tauri-apps/plugin-os";

/** System preview is backed by Quick Look on macOS and the Shell preview host on Windows. */
export function isSystemPreviewAvailable(): boolean {
  return platform() === "macos" || platform() === "windows";
}

/**
 * Previews a file in the system panel. The path must be a Space-owned asset: Rust re-authorizes it
 * against the private asset directory, so the WebView cannot turn this into a general file reader.
 */
export function openSystemPreview(path: string): Promise<void> {
  if (!isSystemPreviewAvailable()) {
    return Promise.reject(new Error("System preview is unavailable on this platform"));
  }
  return invoke("plugin:spaces|preview_asset", { sourcePath: path });
}
