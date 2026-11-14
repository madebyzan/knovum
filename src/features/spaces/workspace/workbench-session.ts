import { z } from "zod";

import { isSpaceEditorTab, type SpaceWorkbenchTab } from "./workbench-tab";

const WORKBENCH_SESSION_KEY = "knovum.workspace.workbench";

const savedTabSchema = z.discriminatedUnion("kind", [
  z.object({ kind: z.literal("space"), spaceId: z.string().min(1) }).strict(),
  z
    .object({ kind: z.enum(["document", "video"]), spaceId: z.string().min(1), nodeId: z.string().min(1) })
    .strict(),
]);

const workbenchSessionSchema = z
  .object({
    tabs: z.array(savedTabSchema),
    activeIndex: z.number().int().nonnegative().nullable(),
  })
  .strict()
  .refine(({ tabs, activeIndex }) => activeIndex === null || activeIndex < tabs.length);

/** One tab as the next launch reopens it: what it shows, never its session-scoped id. */
export type SavedWorkbenchTab = z.infer<typeof savedTabSchema>;

/**
 * The tab strip as the window last showed it. It is disposable navigation state: it never holds
 * content, and the Spaces and nodes it names are revalidated when it is restored.
 */
export interface WorkbenchSession {
  readonly tabs: readonly SavedWorkbenchTab[];
  readonly activeIndex: number | null;
}

/**
 * Hands the saved session to exactly one restore. Taking it consumes it, so a restore that fails
 * cannot fail again on the next attempt; the restored workbench saves itself once it is ready.
 */
export function takeWorkbenchSession(): WorkbenchSession | undefined {
  const value = globalThis.localStorage.getItem(WORKBENCH_SESSION_KEY);
  if (value === null) return undefined;
  globalThis.localStorage.removeItem(WORKBENCH_SESSION_KEY);
  try {
    const parsed = workbenchSessionSchema.safeParse(JSON.parse(value));
    return parsed.success ? parsed.data : undefined;
  } catch {
    return undefined;
  }
}

/** Saves the strip as the next launch should reopen it, leaving out the tabs of an unsaved draft. */
export function saveWorkbenchSession(
  tabs: readonly SpaceWorkbenchTab[],
  activeTabId: string | undefined,
  draftSpaceId: string | undefined,
): void {
  const saved = tabs.filter(({ spaceId }) => spaceId !== draftSpaceId);
  const activeIndex = saved.findIndex(({ id }) => id === activeTabId);
  const session: WorkbenchSession = {
    tabs: saved.map(savedTab),
    activeIndex: activeIndex < 0 ? null : activeIndex,
  };
  globalThis.localStorage.setItem(WORKBENCH_SESSION_KEY, JSON.stringify(session));
}

function savedTab(tab: SpaceWorkbenchTab): SavedWorkbenchTab {
  return isSpaceEditorTab(tab)
    ? { kind: tab.kind, spaceId: tab.spaceId, nodeId: tab.nodeId }
    : { kind: tab.kind, spaceId: tab.spaceId };
}
