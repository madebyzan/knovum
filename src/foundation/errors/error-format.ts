import { decodeAppError, normalizeError } from "./app-error";

export function errorMessage(reason: unknown): string {
  return normalizeError(reason).message;
}

export function errorDiagnostic(reason: unknown): string {
  return describe(reason, new Set());
}

function describe(reason: unknown, seen: Set<unknown>): string {
  const payload = decodeAppError(reason);
  if (!(reason instanceof Error)) {
    if (payload) return `[${payload.code}] ${payload.message}`;
    return typeof reason === "string" ? reason : "Unrecognized error payload";
  }
  if (seen.has(reason)) return "[Circular error cause]";
  seen.add(reason);
  const details = [reason.stack ?? `${reason.name}: ${reason.message}`];
  if (payload) details.push(`Code: ${payload.code}`);
  else if (reason.cause !== undefined) details.push(`Cause: ${describe(reason.cause, seen)}`);
  if (reason instanceof AggregateError) {
    details.push(...reason.errors.map((error: unknown) => describe(error, seen)));
  }
  seen.delete(reason);
  return details.join("\n");
}
