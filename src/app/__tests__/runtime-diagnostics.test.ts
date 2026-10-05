import { describe, expect, it, vi } from "vitest";

import { installRuntimeDiagnostics, isResizeObserverNotification } from "../runtime-diagnostics";

describe("runtime diagnostics", () => {
  it("ignores only the browser ResizeObserver delivery notifications", () => {
    expect(isResizeObserverNotification("ResizeObserver loop limit exceeded")).toBe(true);
    expect(
      isResizeObserverNotification("ResizeObserver loop completed with undelivered notifications."),
    ).toBe(true);
    expect(isResizeObserverNotification("DataCloneError: The object can not be cloned.")).toBe(false);
  });

  it("registers diagnostic listeners once across bootstrap retries", () => {
    const listen = vi.spyOn(globalThis, "addEventListener").mockImplementation(() => {});
    installRuntimeDiagnostics();
    installRuntimeDiagnostics();
    expect(listen).toHaveBeenCalledTimes(2);
  });
});
