import { delay } from "es-toolkit";

import { invokeCommand as invoke } from "../../../foundation/ipc/invoke-command";

const CONFIRMATION_DELAYS_MS = [0, 250, 750] as const;

/** Resolve an ambiguous IPC failure from the committed mutation receipt before reporting failure. */
export async function invokeConfirmedMutation<T>(
  command: string,
  args: Record<string, unknown>,
  mutationId: string,
): Promise<T> {
  try {
    return await invoke<T>(command, args);
  } catch (error) {
    // Returned command/dispatcher rejections carry their native payload as cause.
    // Do not replace an explicit failure with an older request's receipt.
    if (error instanceof Error && error.cause !== undefined) throw error;
    const committed = await confirmReceipt<T>(mutationId, 0);
    if (committed !== null) return committed;
    throw error;
  }
}

async function confirmReceipt<T>(mutationId: string, attempt: number): Promise<T | null> {
  const waitMs = CONFIRMATION_DELAYS_MS[attempt];
  if (waitMs === undefined) return null;
  if (waitMs) await delay(waitMs);
  const committed = await invoke<T | null>("plugin:spaces|get_space_edit_receipt", {
    mutationId,
  }).catch(() => null);
  return committed ?? confirmReceipt<T>(mutationId, attempt + 1);
}
