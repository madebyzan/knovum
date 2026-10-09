import type { Router } from "vue-router";
import { z } from "zod";

const LAST_ROUTE_KEY = "knovum.workspace.lastRoute";
const LAST_NON_SETTINGS_ROUTE_KEY = "knovum.workspace.lastNonSettingsRoute";
const routeSchema = z
  .string()
  .startsWith("/")
  .refine((path) => !path.startsWith("//"));

function readRoute(key: string): string | undefined {
  const value = globalThis.localStorage.getItem(key);
  if (value === null) return undefined;

  const parsed = routeSchema.safeParse(value);
  if (parsed.success) return parsed.data;

  globalThis.localStorage.removeItem(key);
  return undefined;
}

export async function returnToWorkspace(router: Router): Promise<void> {
  const path = readRoute(LAST_NON_SETTINGS_ROUTE_KEY);
  await router.push(path && router.resolve(path).name === "spaces" ? path : "/spaces");
}

export async function restoreWorkspaceSession(router: Router): Promise<void> {
  const path = readRoute(LAST_ROUTE_KEY);
  if (!path || router.resolve(path).matched.length === 0) return;
  await router.replace(path);
}

export function installWorkspaceSessionPersistence(router: Router): void {
  router.afterEach((to) => {
    globalThis.localStorage.setItem(LAST_ROUTE_KEY, to.fullPath);
    if (to.name === "spaces") {
      globalThis.localStorage.setItem(LAST_NON_SETTINGS_ROUTE_KEY, to.fullPath);
    }
  });
}
