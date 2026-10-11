import { expect, it, onTestFinished, vi } from "vitest";

import { createEditorSurfaceActions } from "../workspace/editor-surface-actions";
import { registerSpaceSessionParticipant } from "../workspace/session-participants";
import { activateTestEditor, createTestSessionContext } from "./support/session-context";

it.each(["saved", "failed"])("returns to canvas only after its session is %s", async (outcome) => {
  const context = createTestSessionContext();
  const tab = activateTestEditor(context, "document", "note");
  const failure = new Error("disk full");
  const persist = vi.fn<() => Promise<void>>();
  if (outcome === "saved") persist.mockResolvedValue();
  else persist.mockRejectedValue(failure);
  onTestFinished(
    registerSpaceSessionParticipant({
      editorOwner: { spaceId: tab.spaceId, tabId: tab.id, commitOnSwitch: true },
      commit: persist,
    }),
  );
  const result = await createEditorSurfaceActions(context)
    .returnToCanvas()
    .then(
      () => undefined,
      (error: unknown) => error,
    );
  expect(result).toBe(outcome === "saved" ? undefined : failure);
  expect(context.surface.value.kind).toBe(outcome === "saved" ? "canvas" : "document");
  expect(persist).toHaveBeenCalledOnce();
  expect(context.editorTabs.value).toHaveLength(1);
});
