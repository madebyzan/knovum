import { afterEach, describe, expect, it } from "vitest";
import { createMemoryHistory, createRouter } from "vue-router";

import {
  installWorkspaceSessionPersistence,
  readLastNonSettingsRoute,
  restoreWorkspaceSession,
} from "../../foundation/navigation/workspace-session";

function createTestRouter() {
  return createRouter({
    history: createMemoryHistory(),
    routes: [
      { path: "/spaces", component: { template: "<div />" } },
      { path: "/settings", component: { template: "<div />" } },
    ],
  });
}

afterEach(() => localStorage.clear());

describe("workspace session persistence", () => {
  it("persists the full route while keeping the last non-settings destination", async () => {
    const router = createTestRouter();
    installWorkspaceSessionPersistence(router);

    await router.push("/spaces?view=canvas");
    expect(readLastNonSettingsRoute()).toBe("/spaces?view=canvas");

    await router.push("/settings?pane=appearance");
    expect(readLastNonSettingsRoute()).toBe("/spaces?view=canvas");
    expect(localStorage.getItem("knovum.workspace.lastRoute")).toBe("/settings?pane=appearance");
  });

  it("restores query state instead of dropping it", async () => {
    localStorage.setItem("knovum.workspace.lastRoute", "/settings?pane=aiModels");
    const router = createTestRouter();

    await restoreWorkspaceSession(router);

    expect(router.currentRoute.value.fullPath).toBe("/settings?pane=aiModels");
  });
});
