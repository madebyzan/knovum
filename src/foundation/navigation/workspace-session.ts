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

export function readLastNonSettingsRoute(): string | undefined {
  return readRoute(LAST_NON_SETTINGS_ROUTE_KEY);
}

export async function restoreWorkspaceSession(router: Router): Promise<void> {
  const path = readRoute(LAST_ROUTE_KEY);
  if (!path || router.resolve(path).matched.length === 0) return;
  await router.replace(path);
}

export function installWorkspaceSessionPersistence(router: Router): void {
  router.afterEach((to) => {
    globalThis.localStorage.setItem(LAST_ROUTE_KEY, to.fullPath);
    if (to.path !== "/settings") {
      globalThis.localStorage.setItem(LAST_NON_SETTINGS_ROUTE_KEY, to.fullPath);
    }
  });
}
