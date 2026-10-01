import { error as logError } from "@tauri-apps/plugin-log";

import { normalizeError } from "../foundation/errors/app-error";
import { errorDiagnostic } from "../foundation/errors/error-format";

export function handleBoundaryError(caught: unknown, prefix?: string): Error {
  const error = normalizeError(caught);
  const diagnostic = errorDiagnostic(error);
  void logError(prefix ? `[${prefix}] ${diagnostic}` : diagnostic);
  return error;
}
