import { error as logError } from "@tauri-apps/plugin-log";

export function handleBoundaryError(caught: unknown, prefix?: string): Error {
  const error = caught instanceof Error ? caught : new Error(String(caught));
  const diagnosticText = error.stack ? `${error.message}\n${error.stack}` : error.message;
  const message = prefix ? `[${prefix}] ${diagnosticText}` : (error.stack ?? error.message);
  void logError(message);
  return error;
}
