import { decodeAppError, normalizeError } from "./app-error";

export function getErrorMessage(reason: unknown): string {
  return normalizeError(reason).message;
}

export function formatErrorDiagnostic(reason: unknown): string {
  return describe(reason, new Set());
}

function describe(reason: unknown, seen: Set<Error>): string {
  if (typeof reason === "string") return reason;
  if (!(reason instanceof Error)) {
    const payload = decodeAppError(reason);
    return payload ? `[${payload.code}] ${payload.message}` : "Unrecognized error payload";
  }
  if (seen.has(reason)) return "[Circular error cause]";
  seen.add(reason);
  const diagnosticLines = [reason.stack ?? `${reason.name}: ${reason.message}`];
  if (reason.cause !== undefined) diagnosticLines.push(`Cause: ${describe(reason.cause, seen)}`);
  if (reason instanceof AggregateError) {
    diagnosticLines.push(...reason.errors.map((error: unknown) => describe(error, seen)));
  }
  seen.delete(reason);
  return diagnosticLines.join("\n");
}
