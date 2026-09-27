import { invoke } from "@tauri-apps/api/core";

import { normalizeCommandError } from "./command-error";

export async function invokeCommand<T>(...args: Parameters<typeof invoke>): Promise<T> {
  try {
    return await invoke<T>(...args);
  } catch (cause) {
    throw normalizeCommandError(cause);
  }
}
