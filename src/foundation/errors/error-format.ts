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
  const details = [reason.stack ?? `${reason.name}: ${reason.message}`];
  if (reason.cause !== undefined) details.push(`Cause: ${describe(reason.cause, seen)}`);
  if (reason instanceof AggregateError) {
    details.push(...reason.errors.map((error: unknown) => describe(error, seen)));
  }
  seen.delete(reason);
  return details.join("\n");
}
