import { getCurrentWebview } from "@tauri-apps/api/webview";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { error as logError } from "@tauri-apps/plugin-log";
import { platform, type Platform } from "@tauri-apps/plugin-os";

import { formatErrorDiagnostic } from "../errors/error-format";

export interface DragDropPosition {
  x: number;
  y: number;
}

export type FileDrop = { paths: string[]; position: DragDropPosition };

export function isFileDropInside(
  element: HTMLElement | null | undefined,
  position: DragDropPosition | undefined,
) {
  if (!element || !position) return false;
  const rect = element.getBoundingClientRect();
  return (
    position.x >= rect.left && position.x <= rect.right && position.y >= rect.top && position.y <= rect.bottom
  );
}

export interface FileDropHandlers {
  hover?(active: boolean, position?: DragDropPosition): void;
  drop(payload: FileDrop): void | Promise<void>;
}

/**
 * Normalizes window-server drag coordinates into CSS viewport coordinates.
 * AppKit (macOS) and WebKitGTK (Linux) report logical viewport coordinates,
 * while WebView2 (Windows) reports physical device pixels.
 */
export function normalizeDragPosition(
  position: DragDropPosition,
  currentPlatform: Platform,
  scaleFactor: number,
): DragDropPosition {
  if (currentPlatform !== "windows") return position;
  return { x: position.x / scaleFactor, y: position.y / scaleFactor };
}

const NOOP_DISPOSER = () => {};

/**
 * Listens for Tauri Webview drag-and-drop events with viewport-normalized coordinates.
 * Automatically synchronizes display scale factor changes on Windows (WebView2).
 */
export async function listenForFileDrops(handlers: FileDropHandlers): Promise<() => void> {
  const window = getCurrentWindow();
  const currentPlatform = platform();
  let scaleFactor = 1;
  let unlistenScale = NOOP_DISPOSER;

  if (currentPlatform === "windows") {
    scaleFactor = await window.scaleFactor();
    unlistenScale = await window.onScaleChanged(({ payload }) => {
      scaleFactor = payload.scaleFactor;
    });
  }

  let hasFilePayload = false;
  const emitHover = (position: DragDropPosition) =>
    handlers.hover?.(true, normalizeDragPosition(position, currentPlatform, scaleFactor));
  const emitDrop = (paths: string[], position: DragDropPosition) => {
    const drop: FileDrop = {
      paths,
      position: normalizeDragPosition(position, currentPlatform, scaleFactor),
    };
    void Promise.resolve()
      .then(() => handlers.drop(drop))
      .catch((err) => logError(`File drop handler failed: ${formatErrorDiagnostic(err)}`));
  };

  const unlistenDrop = await getCurrentWebview().onDragDropEvent(({ payload }) => {
    if (payload.type === "enter") {
      hasFilePayload = payload.paths.length > 0;
      if (hasFilePayload) emitHover(payload.position);
      return;
    }
    if (payload.type === "over") {
      if (hasFilePayload) emitHover(payload.position);
      return;
    }
    hasFilePayload = false;
    handlers.hover?.(false);
    if (payload.type === "drop" && payload.paths.length) emitDrop(payload.paths, payload.position);
  });

  return () => {
    unlistenDrop();
    unlistenScale();
  };
}
