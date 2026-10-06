import { $, browser, expect } from "@wdio/globals";

import { runAppCommand } from "./app-command";

async function openCommandPalette(palette: ReturnType<typeof $>, attempts = 10): Promise<void> {
  if (attempts === 0 || (await palette.isDisplayed())) return;
  await runAppCommand("shell.command-palette");
  if (!(await palette.isDisplayed())) await browser.pause(50);
  await openCommandPalette(palette, attempts - 1);
}

describe("desktop shell", () => {
  it("opens the command palette through the application menu command channel", async () => {
    await expect($("[data-testid='spaces-workspace']")).toBeDisplayed();
    const palette = $("[data-testid='command-palette-input']");

    await openCommandPalette(palette);

    await expect(palette).toBeDisplayed();
    expect((await browser.getTitle()).startsWith("Knovum")).toBe(true);
    await browser.keys("Escape");
    await expect(palette).not.toBeDisplayed();
  });

  it("returns from Settings to the persistent knowledge workspace", async () => {
    await runAppCommand("navigate.settings");
    await expect($("[data-testid='settings-view']")).toBeDisplayed();
    await $("[data-testid='settings-pane-usage']").click();
    await expect($("[data-testid='usage-settings-pane']")).toBeDisplayed();
    await expect($("[data-testid='token-activity-calendar']")).toBeDisplayed();
    await $("[data-testid='settings-back']").click();
    await expect($("[data-testid='spaces-workspace']")).toBeDisplayed();
  });
});
