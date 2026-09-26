import { invoke, type InvokeArgs, type InvokeOptions } from "@tauri-apps/api/core";

import { normalizeCommandError } from "./command-error";

export async function invokeCommand<T>(
  command: string,
  args?: InvokeArgs,
  options?: InvokeOptions,
): Promise<T> {
  try {
    if (options !== undefined) return await invoke<T>(command, args, options);
    if (args !== undefined) return await invoke<T>(command, args);
    return await invoke<T>(command);
  } catch (cause) {
    throw normalizeCommandError(cause);
  }
}
