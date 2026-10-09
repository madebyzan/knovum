import { $, browser, expect } from "@wdio/globals";

type TauriGlobal = {
  core?: { invoke(command: string, payload?: Record<string, unknown>): Promise<unknown> };
};

async function invoke(command: string): Promise<string | null> {
  return (await browser.executeAsync(async (address, done) => {
    const tauri = (globalThis as typeof globalThis & { __TAURI__?: TauriGlobal }).__TAURI__;
    if (!tauri?.core?.invoke) {
      done("withGlobalTauri is unavailable in this build");
      return;
    }
    try {
      await tauri.core.invoke(address);
      done(null);
    } catch (error) {
      done(String(error));
    }
  }, command)) as string | null;
}

describe("in-process plugin IPC surface", () => {
  it("waits for the bootstrap to reveal the shell", async () => {
    await expect($("[data-testid='workspace']")).toBeDisplayed();
  });

  it("denies an undeclared command inside the plugin namespace", async () => {
    expect(await invoke("plugin:spaces|undeclared")).toMatch(/not allowed|not found|unknown/i);
  });
});
