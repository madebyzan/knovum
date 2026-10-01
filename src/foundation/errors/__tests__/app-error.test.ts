import { expect, it } from "vitest";

import { createAppError, decodeAppError, normalizeError } from "../app-error";
import { formatErrorDiagnostic, getErrorMessage } from "../error-format";

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
    expect(getErrorMessage(error)).toBe("Failure detail");
    expect(formatErrorDiagnostic(error)).toContain(code);
  }
  expect(decodeAppError(createAppError("cancelled", "Cancelled"))?.code).toBe("cancelled");
});

it("preserves native error identity and reports vendor strings at the boundary", () => {
  const error = new TypeError("Invalid input");
  expect(normalizeError(error)).toBe(error);
  expect(getErrorMessage(error)).toBe("Invalid input");
  expect(formatErrorDiagnostic(error)).toContain(error.stack);
  expect(getErrorMessage("Permission denied")).toBe("Permission denied");
});

it("formats structured payloads and vendor strings directly", () => {
  expect(formatErrorDiagnostic({ code: "cancelled", message: "Cancelled" })).toBe("[cancelled] Cancelled");
  expect(formatErrorDiagnostic("Permission denied")).toBe("Permission denied");
});

it("retains both the outer error and the structured cause details", () => {
  const error = new Error("Import failed", {
    cause: { code: "permission_denied", message: "Source file is not accessible" },
  });
  const diagnostic = formatErrorDiagnostic(error);
  expect(diagnostic).toContain(error.stack);
  expect(diagnostic).toContain("[permission_denied] Source file is not accessible");
});

it("retains nested native errors and their classified causes", () => {
  const cause = createAppError("operation_failed", "Read failed");
  const diagnostic = formatErrorDiagnostic(new Error("Import failed", { cause }));
  expect(diagnostic).toContain("Import failed");
  expect(diagnostic).toContain(cause.stack);
  expect(diagnostic).toContain("operation_failed");
});

it("rejects malformed product payloads without guessing codes or fields", () => {
  const malformed = { code: "old-code", message: "Detail", obsolete: true };
  const error = normalizeError(malformed);
  expect(error.cause).toBe(malformed);
  expect(decodeAppError(error)).toBeNull();
  expect(getErrorMessage(error)).toBe("Unexpected error");
});

it("keeps causes and all aggregate failures available in diagnostics", () => {
  const primary = createAppError("operation_failed", "Import failed");
  const cleanup = new Error("Cleanup failed");
  const error = new AggregateError([primary, cleanup], "Import could not complete", { cause: primary });
  expect(getErrorMessage(error)).toBe("Import could not complete");
  expect(formatErrorDiagnostic(error)).toContain("Import failed");
  expect(formatErrorDiagnostic(error)).toContain("Cleanup failed");
  expect(formatErrorDiagnostic(error)).toContain("operation_failed");
  expect(formatErrorDiagnostic(error)).not.toContain("[Circular error cause]");
});

it("bounds diagnostics for a circular native error cause", () => {
  const error = new Error("Failed");
  error.cause = error;
  expect(formatErrorDiagnostic(error)).toContain("[Circular error cause]");
});

it("does not execute arbitrary object stringification while reporting a malformed rejection", () => {
  const reason = {
    toString() {
      throw new Error("Must not be called");
    },
  };
  const error = normalizeError(reason);
  expect(error.cause).toBe(reason);
  expect(formatErrorDiagnostic(error)).toContain("Unrecognized error payload");
});
