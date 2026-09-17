import { describe, expect, it } from "vitest";

import { isResizeObserverNotification } from "../runtime-diagnostics";

describe("runtime diagnostics", () => {
  it("ignores only the browser ResizeObserver delivery notifications", () => {
    expect(isResizeObserverNotification("ResizeObserver loop limit exceeded")).toBe(true);
    expect(
      isResizeObserverNotification("ResizeObserver loop completed with undelivered notifications."),
    ).toBe(true);
    expect(isResizeObserverNotification("DataCloneError: The object can not be cloned.")).toBe(false);
  });
});
