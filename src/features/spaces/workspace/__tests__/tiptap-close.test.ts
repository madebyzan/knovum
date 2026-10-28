import { expect, it, onTestFinished, vi } from "vitest";

import { activateTestEditor, createTestSessionContext } from "../../__tests__/support/session-context";
import { registerSpaceSessionParticipant } from "../session-participants";
import { openSpaceCanvas } from "../workbench-navigation-actions";

it.each(["saved", "failed"])("returns to canvas only after its session is %s", async (outcome) => {
  const context = createTestSessionContext();
  const tab = activateTestEditor(context, "document", "note");
  const failure = new Error("disk full");
  const persist = vi.fn<() => Promise<void>>();
  if (outcome === "saved") persist.mockResolvedValue();
  else persist.mockRejectedValue(failure);
  onTestFinished(
    registerSpaceSessionParticipant({
      editorOwner: { spaceId: tab.spaceId, tabId: tab.id },
      commit: persist,
    }),
  );
  const leaveFailure = await openSpaceCanvas(context).then(
    () => undefined,
    (error: unknown) => error,
  );
  expect(leaveFailure).toBe(outcome === "saved" ? undefined : failure);
  expect(context.surface.value.kind).toBe(outcome === "saved" ? "canvas" : "document");
  expect(persist).toHaveBeenCalledOnce();
  expect(context.editorTabs.value).toHaveLength(1);
});
