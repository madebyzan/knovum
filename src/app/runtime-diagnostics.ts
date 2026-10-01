import { error } from "@tauri-apps/plugin-log";
import { useEventListener } from "@vueuse/core";

import { formatErrorDiagnostic } from "../foundation/errors/error-format";

export function installRuntimeDiagnostics(): void {
  useEventListener(globalThis, "error", (event: ErrorEvent) => {
    const reason = event.error ?? event.message;
    if (!isResizeObserverNotification(reason)) void error(`window.error: ${formatErrorDiagnostic(reason)}`);
  });

  useEventListener(globalThis, "unhandledrejection", (event: PromiseRejectionEvent) => {
    void error(`unhandledrejection: ${formatErrorDiagnostic(event.reason)}`);
  });
}

export function isResizeObserverNotification(reason: unknown): boolean {
  return (
    reason === "ResizeObserver loop limit exceeded" ||
    reason === "ResizeObserver loop completed with undelivered notifications."
  );
}
