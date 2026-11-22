import { z } from "zod";

import { savedSpaceViewSchema, type SavedSpaceViewSession } from "./space-view-session";
import { isSpaceEditorTab, type SpaceWorkbenchTab } from "./workbench-tab";

const WORKBENCH_SESSION_KEY = "knovum.workspace.workbench";
const nonEmptyId = z.string().min(1);

const savedTabSchema = z.discriminatedUnion("kind", [
  z.object({ kind: z.literal("space"), spaceId: nonEmptyId }).strict(),
  z.object({ kind: z.enum(["document", "video"]), spaceId: nonEmptyId, nodeId: nonEmptyId }).strict(),
]);

const workbenchSessionSchema = z
  .object({
    tabs: z.array(savedTabSchema),
    activeIndex: z.number().int().nonnegative().nullable(),
    spaceViews: z.record(nonEmptyId, savedSpaceViewSchema),
  })
  .strict()
  .refine(({ tabs, activeIndex }) => activeIndex === null || activeIndex < tabs.length);

/** One tab as the next launch reopens it: what it shows, never its session-scoped id. */
export type SavedWorkbenchTab = z.infer<typeof savedTabSchema>;

/**
 * The workbench session as the window last showed it. It is disposable navigation and view state:
 * it never holds document/media content, and every Space, node, group and Saved View it names is
 * revalidated when it is restored.
 */
export interface WorkbenchSession {
  readonly tabs: readonly SavedWorkbenchTab[];
  readonly activeIndex: number | null;
  readonly spaceViews: Readonly<Record<string, SavedSpaceViewSession>>;
}

export interface SaveWorkbenchSessionInput {
  readonly tabs: readonly SpaceWorkbenchTab[];
  readonly activeTabId: string | undefined;
  readonly draftSpaceId: string | undefined;
  readonly spaceViews: ReadonlyMap<string, SavedSpaceViewSession>;
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

/** Saves the workbench session as the next launch should reopen it, omitting any unsaved draft Space. */
export function saveWorkbenchSession(input: SaveWorkbenchSessionInput): void {
  const saved = input.tabs.filter(({ spaceId }) => spaceId !== input.draftSpaceId);
  const activeIndex = saved.findIndex(({ id }) => id === input.activeTabId);
  const persistedSpaceIds = new Set(saved.map(({ spaceId }) => spaceId));
  const spaceViews: Record<string, SavedSpaceViewSession> = {};
  for (const [spaceId, view] of input.spaceViews) {
    if (persistedSpaceIds.has(spaceId)) spaceViews[spaceId] = view;
  }
  const session: WorkbenchSession = {
    tabs: saved.map(savedTab),
    activeIndex: activeIndex < 0 ? null : activeIndex,
    spaceViews,
  };
  globalThis.localStorage.setItem(WORKBENCH_SESSION_KEY, JSON.stringify(session));
}

function savedTab(tab: SpaceWorkbenchTab): SavedWorkbenchTab {
  return isSpaceEditorTab(tab)
    ? { kind: tab.kind, spaceId: tab.spaceId, nodeId: tab.nodeId }
    : { kind: tab.kind, spaceId: tab.spaceId };
}
