import { error } from "@tauri-apps/plugin-log";
import { useEventListener } from "@vueuse/core";

function describe(reason: unknown): string {
  if (reason instanceof Error) return reason.stack ?? reason.message;
  return String(reason);
}

export function installRuntimeDiagnostics(): void {
  useEventListener(globalThis, "error", (event: ErrorEvent) => {
    const reason = event.error ?? event.message;
    if (!isResizeObserverNotification(reason)) void error(`window.error: ${describe(reason)}`);
  });

  useEventListener(globalThis, "unhandledrejection", (event: PromiseRejectionEvent) => {
    void error(`unhandledrejection: ${describe(event.reason)}`);
  });
}

export function isResizeObserverNotification(reason: unknown): boolean {
  return (
    reason === "ResizeObserver loop limit exceeded" ||
    reason === "ResizeObserver loop completed with undelivered notifications."
  );
}
