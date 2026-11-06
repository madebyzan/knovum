import { invokeCommand } from "../../../foundation/ipc/invoke-command";

/**
 * Shows a file the way the platform shows content: Quick Look on macOS, the associated application
 * elsewhere. The path must be a Space-owned asset: Rust re-authorizes it against the private asset
 * directory, so the WebView cannot turn this into a general file reader.
 */
export function previewSpaceAsset(path: string): Promise<void> {
  return invokeCommand("plugin:spaces|preview_asset", { sourcePath: path });
}
