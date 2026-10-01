import { expect, it } from "vitest";

import { createAppError, decodeAppError, normalizeError } from "../app-error";
import { errorDiagnostic, errorMessage } from "../error-format";

it("uses the same contract for AI, native commands and local cancellation", () => {
  for (const code of [
    "not_configured",
    "authentication_failed",
    "space_edit_revision_conflict",
    "cancelled",
  ]) {
    const error = normalizeError({ code, message: "Failure detail" });
    expect(error).toBeInstanceOf(Error);
    expect(decodeAppError(error)).toEqual({ code, message: "Failure detail" });
    expect(errorMessage(error)).toBe("Failure detail");
    expect(errorDiagnostic(error)).toContain(code);
  }
  expect(decodeAppError(createAppError("cancelled", "Cancelled"))?.code).toBe("cancelled");
});

it("preserves native error identity and reports vendor strings at the boundary", () => {
  const error = new TypeError("Invalid input");
  expect(normalizeError(error)).toBe(error);
  expect(errorMessage(error)).toBe("Invalid input");
  expect(errorDiagnostic(error)).toContain(error.stack);
  expect(errorMessage("Permission denied")).toBe("Permission denied");
});

it("rejects malformed product payloads without guessing codes or fields", () => {
  const malformed = { code: "old-code", message: "Detail", obsolete: true };
  const error = normalizeError(malformed);
  expect(error.cause).toBe(malformed);
  expect(decodeAppError(error)).toBeNull();
  expect(errorMessage(error)).toBe("Unexpected error");
});

it("keeps causes and all aggregate failures available in diagnostics", () => {
  const primary = createAppError("operation_failed", "Import failed");
  const cleanup = new Error("Cleanup failed");
  const error = new AggregateError([primary, cleanup], "Import could not complete", { cause: primary });
  expect(errorMessage(error)).toBe("Import could not complete");
  expect(errorDiagnostic(error)).toContain("Import failed");
  expect(errorDiagnostic(error)).toContain("Cleanup failed");
  expect(errorDiagnostic(error)).toContain("operation_failed");
  expect(errorDiagnostic(error)).not.toContain("[Circular error cause]");
});

it("bounds diagnostics for a circular native error cause", () => {
  const error = new Error("Failed");
  error.cause = error;
  expect(errorDiagnostic(error)).toContain("[Circular error cause]");
});

it("does not execute arbitrary object stringification while reporting a malformed rejection", () => {
  const reason = {
    toString() {
      throw new Error("Must not be called");
    },
  };
  const error = normalizeError(reason);
  expect(error.cause).toBe(reason);
  expect(errorDiagnostic(error)).toContain("Unrecognized error payload");
});
