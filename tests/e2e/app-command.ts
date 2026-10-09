import { browser, expect } from "@wdio/globals";

export async function runAppCommand(commandId: string): Promise<void> {
  const outcome = (await browser.executeAsync(async (commandId, done) => {
    const bridge = (
      globalThis as typeof globalThis & {
        __knovumE2eBridge?: { runCommand(commandId: string): Promise<void> };
      }
    ).__knovumE2eBridge;
    if (!bridge) {
      done({ error: "E2E command bridge is unavailable" });
      return;
    }
    try {
      await bridge.runCommand(commandId);
      done({ error: null });
    } catch (error) {
      done({ error: String(error) });
    }
  }, commandId)) as { error: string | null };
  expect(outcome.error).toBeNull();
}
