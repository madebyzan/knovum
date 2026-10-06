import { randomUUID } from "node:crypto";
import { mkdtemp, mkdir, writeFile, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";

import { $, browser, expect } from "@wdio/globals";

import type { PluginInstallation } from "../../src/shared/contracts/generated/PluginInstallation";
import type { PluginMarketplace } from "../../src/shared/contracts/generated/PluginMarketplace";
import { runAppCommand } from "./app-command";
import { extensionCommand } from "./extension-ipc";

describe("Agent extension installation lifecycle", () => {
  it("discovers, reviews, installs, disables, updates and uninstalls an immutable local package", async () => {
    const root = await mkdtemp(join(tmpdir(), "knovum-extension-e2e-"));
    const name = `e2e-${randomUUID()}`;
    const packagePath = join(root, "package");
    const skillPath = join(packagePath, "skills", name);
    await mkdir(skillPath, { recursive: true });
    const manifest = {
      $schema: "https://agent-plugins.org/schemas/1.0.0/plugin.schema.json",
      name,
      version: "reviewed-one",
    };
    await writeFile(join(packagePath, "plugin.json"), JSON.stringify(manifest));
    await writeFile(
      join(skillPath, "SKILL.md"),
      `---\nname: ${name}\ndescription: Local lifecycle fixture\n---\nReviewed instructions.`,
    );
    await writeFile(
      join(root, "marketplace.json"),
      JSON.stringify({
        schemaVersion: 1,
        name,
        displayName: name,
        releases: [
          {
            publisherId: name,
            name,
            description: "Local fixture",
            source: { type: "directory", path: "package" },
          },
        ],
      }),
    );
    let installation: PluginInstallation | undefined;
    let marketplace: PluginMarketplace | undefined;
    try {
      await runAppCommand("navigate.settings");
      await $("[data-testid='settings-pane-extensions']").click();
      await expect($("[data-testid='agent-extensions-settings']")).toBeDisplayed();
      await $("[data-testid='marketplace-source-type']").selectByAttribute("value", "directory");
      await $("[data-testid='marketplace-location']").setValue(root);
      await $("[data-testid='add-marketplace']").click();
      await browser.waitUntil(async () => {
        marketplace = (await extensionCommand<PluginMarketplace[]>("search_plugins", { query: name })).find(
          (item) => item.name === name,
        );
        return Boolean(marketplace);
      });
      await $(`[data-testid='preview-plugin-${name}']`).click();
      await expect($("[data-testid='plugin-installation-preview']")).toBeDisplayed();
      await $("[data-testid='confirm-plugin-installation']").click();
      await browser.waitUntil(async () => {
        installation = (await extensionCommand<PluginInstallation[]>("list_plugin_installations")).find(
          (item) => item.revision.manifest.name === name,
        );
        return Boolean(installation);
      });
      const installed = installation!;
      const firstDigest = installed.revision.digest;
      await expect($(`[data-testid='plugin-${installed.id}']`)).toBeDisplayed();
      await $(`[data-testid='toggle-plugin-${installed.id}']`).click();
      await browser.waitUntil(
        async () =>
          !(await extensionCommand<PluginInstallation[]>("list_plugin_installations")).find(
            (item) => item.id === installed.id,
          )!.enabled,
      );
      await writeFile(
        join(packagePath, "plugin.json"),
        JSON.stringify({ ...manifest, version: "reviewed-two" }),
      );
      await $(`[data-testid='update-plugin-${installed.id}']`).click();
      await expect($("[data-testid='plugin-installation-preview']")).toBeDisplayed();
      await $("[data-testid='confirm-plugin-installation']").click();
      await browser.waitUntil(async () => {
        const current = (await extensionCommand<PluginInstallation[]>("list_plugin_installations")).find(
          (item) => item.id === installed.id,
        )!;
        return (
          current.revision.manifest.version === "reviewed-two" &&
          current.revision.digest !== firstDigest &&
          !current.enabled
        );
      });
      await $(`[data-testid='uninstall-plugin-${installed.id}']`).click();
      await browser.waitUntil(
        async () =>
          !(await extensionCommand<PluginInstallation[]>("list_plugin_installations")).some(
            (item) => item.id === installed.id,
          ),
      );
      installation = undefined;
    } finally {
      if (installation) await extensionCommand("uninstall_plugin", { id: installation.id });
      if (marketplace) await extensionCommand("remove_plugin_marketplace", { id: marketplace.id });
      await rm(root, { recursive: true, force: true });
    }
  });
});
