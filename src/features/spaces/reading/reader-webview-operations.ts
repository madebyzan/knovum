import { invokeCommand } from "../../../foundation/ipc/invoke-command";

export interface ReaderWebviewBounds {
  x: number;
  y: number;
  width: number;
  height: number;
}

export type ReaderWebviewNavigation = "back" | "forward" | "reload";

export function openReaderWebview(url: string, bounds: ReaderWebviewBounds) {
  return invokeCommand<void>("plugin:spaces|open_reader_webview", { url, bounds });
}

export function openReaderAssetPreview(spaceId: string, nodeId: string, bounds: ReaderWebviewBounds) {
  return invokeCommand<void>("plugin:spaces|open_reader_asset_preview", { spaceId, nodeId, bounds });
}

export function updateReaderWebviewBounds(bounds: ReaderWebviewBounds, visible: boolean) {
  return invokeCommand<void>("plugin:spaces|update_reader_webview_bounds", { bounds, visible });
}

export function navigateReaderWebview(action: ReaderWebviewNavigation) {
  return invokeCommand<void>("plugin:spaces|navigate_reader_webview", { action });
}

export function closeReaderWebview() {
  return invokeCommand<void>("plugin:spaces|close_reader_webview");
}
