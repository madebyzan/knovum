import { browser } from "@wdio/globals";
type IpcOutcome = { ok: true; value: unknown } | { ok: false; error: string };

export async function extensionCommand<T>(command: string, payload?: Record<string, unknown>): Promise<T> {
  const outcome = await browser.execute(
    async (address, input): Promise<IpcOutcome> => {
      const tauri = (
        globalThis as typeof globalThis & {
          __TAURI__?: {
            core: { invoke(command: string, input?: Record<string, unknown>): Promise<unknown> };
          };
        }
      ).__TAURI__;
      if (!tauri) {
        return { ok: false, error: "E2E Tauri IPC unavailable" };
      }
      try {
        return { ok: true, value: await tauri.core.invoke(address, input) };
      } catch (error) {
        return { ok: false, error: String(error) };
      }
    },
    `plugin:extensions|${command}`,
    payload,
  );
  if (!outcome.ok) throw new Error(outcome.error);
  return outcome.value as T;
}
