import { afterEach, describe, expect, it } from "vitest";
import { createMemoryHistory, createRouter } from "vue-router";

import {
  installWorkspaceSessionPersistence,
  returnToWorkspace,
  restoreWorkspaceSession,
} from "../workspace-session";

function createTestRouter() {
  return createRouter({
    history: createMemoryHistory(),
    routes: [
      { path: "/spaces", name: "spaces", component: { template: "<div />" } },
      { path: "/settings", name: "settings", component: { template: "<div />" } },
    ],
  });
}

afterEach(() => localStorage.clear());

describe("workspace session persistence", () => {
  it("persists the full route while keeping the last non-settings destination", async () => {
    const router = createTestRouter();
    installWorkspaceSessionPersistence(router);

    await router.push("/spaces?view=canvas");
    expect(localStorage.getItem("knovum.workspace.lastNonSettingsRoute")).toBe("/spaces?view=canvas");

    await router.push("/settings?pane=appearance");
    expect(localStorage.getItem("knovum.workspace.lastNonSettingsRoute")).toBe("/spaces?view=canvas");
    expect(localStorage.getItem("knovum.workspace.lastRoute")).toBe("/settings?pane=appearance");
  });

  it("returns from Settings to the remembered workspace query", async () => {
    const router = createTestRouter();
    installWorkspaceSessionPersistence(router);
    await router.push("/spaces?view=canvas");
    await router.push("/settings?pane=appearance");
    await returnToWorkspace(router);
    expect(router.currentRoute.value.fullPath).toBe("/spaces?view=canvas");
  });

  it.each(["/settings?pane=voices", "/removed", "//outside.example"])(
    "returns to the product workspace when the remembered destination is %s",
    async (destination) => {
      localStorage.setItem("knovum.workspace.lastNonSettingsRoute", destination);
      const router = createTestRouter();
      await router.push("/settings");
      await returnToWorkspace(router);
      expect(router.currentRoute.value.fullPath).toBe("/spaces");
    },
  );

  it("restores query state instead of dropping it", async () => {
    localStorage.setItem("knovum.workspace.lastRoute", "/settings?pane=aiModels");
    const router = createTestRouter();

    await restoreWorkspaceSession(router);

    expect(router.currentRoute.value.fullPath).toBe("/settings?pane=aiModels");
  });
});
