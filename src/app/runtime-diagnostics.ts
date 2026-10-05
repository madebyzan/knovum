import { error } from "@tauri-apps/plugin-log";

import { formatErrorDiagnostic } from "../foundation/errors/error-format";

let installed = false;

export function installRuntimeDiagnostics(): void {
  if (installed) return;
  installed = true;
  globalThis.addEventListener("error", (event: ErrorEvent) => {
    const reason = event.error ?? event.message;
    if (!isResizeObserverNotification(reason)) void error(`window.error: ${formatErrorDiagnostic(reason)}`);
  });

  globalThis.addEventListener("unhandledrejection", (event: PromiseRejectionEvent) => {
    void error(`unhandledrejection: ${formatErrorDiagnostic(event.reason)}`);
  });
}

export function isResizeObserverNotification(reason: unknown): boolean {
  return (
    reason === "ResizeObserver loop limit exceeded" ||
    reason === "ResizeObserver loop completed with undelivered notifications."
  );
}
