import { invoke } from "@tauri-apps/api/core";

import { normalizeError } from "../errors/app-error";

export async function invokeCommand<T>(...args: Parameters<typeof invoke>): Promise<T> {
  try {
    return await invoke<T>(...args);
  } catch (cause) {
    throw normalizeError(cause);
  }
}
